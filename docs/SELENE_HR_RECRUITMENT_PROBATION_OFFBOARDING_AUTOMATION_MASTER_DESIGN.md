# Selene HR, Recruitment, Probation, Resignation + Termination Automation Master Design

MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design. No runtime code was changed. This document does not authorize implementation, HR execution, recruitment provider integration, termination execution, employment-state mutation, notices, migrations, packets, adapter edits, client edits, or old-path deletion.

The Payroll/HR repo-truth extraction remains the factual base. Current repo truth does not prove a complete HR runtime engine. This document defines future HR lifecycle automation pending Grand Architecture Reconciliation.

## 1. Purpose

Selene must manage the full employee lifecycle:

- recruitment,
- candidate contact,
- resume collection,
- interview/workflow assistance,
- offer,
- onboarding,
- probation,
- active employment,
- promotion,
- resignation,
- termination,
- retirement,
- offboarding,
- rehire.

Selene should reduce routine manager/HR involvement by guiding employees and candidates directly while protected HR actions remain policy, authority, simulation, and audit controlled.

## 2. Recruitment Automation

Selene can assist recruitment by:

- preparing job ads,
- customizing ads by region/location,
- preparing or posting approved listings where integrations exist,
- searching job sites or talent sources where lawful and policy-approved,
- identifying candidate leads,
- inviting candidates to apply through approved channels,
- collecting resumes,
- collecting phone/email,
- starting candidate conversations,
- screening against approved criteria,
- scheduling interviews,
- sending reminders,
- tracking candidate status.

Example request:

"Selene, recruit a warehouse supervisor in Brisbane."

Future flow:

1. Load PH1.POSITION requirements.
2. Draft job ad through PH1.D/PH1.WRITE.
3. Check market salary/requirements where source-backed or owner-approved.
4. Suggest locations/job sites.
5. Ask management to approve ad.
6. Publish/send only through approved channels.
7. Collect candidates.
8. Rank candidates against approved criteria.
9. Arrange direct candidate communication through PH1.BCAST/DELIVERY.
10. Preserve candidate privacy and audit evidence.

Recruitment controls:

- candidate outreach requires approved lawful channels,
- anti-spam and platform terms must be respected,
- candidate privacy and retention policies must apply,
- screening must use approved criteria only,
- forbidden criteria and protected traits must be blocked,
- ranking explanations must be auditable,
- manual review thresholds must exist for sensitive outcomes.

## 3. Candidate Conversation And Screening

Selene may converse with candidates to collect:

- name,
- contact details,
- resume/CV,
- work eligibility evidence where lawful,
- qualifications/certifications,
- availability,
- location/work preference,
- role-specific questions approved by HR,
- interview availability.

Selene must not ask casual questions about protected traits, age, gender, family status, medical status, religion, nationality, or other sensitive criteria unless lawful, policy-approved, and necessary.

Candidate screening must separate:

- minimum requirements,
- preferred requirements,
- certification proof,
- experience evidence,
- availability,
- salary expectation where allowed,
- right-to-work evidence where lawful,
- human review.

PH1.D/GPT-5.5 may draft questions and summarize candidate evidence. HR owns the candidate/recruitment truth.

## 4. Probation Management

Selene must manage probation continuously.

During probation, Selene tracks:

- task evidence,
- attendance evidence,
- training evidence,
- lateness,
- quality,
- manager feedback,
- customer feedback where relevant,
- warnings/support,
- blocked work,
- improvement trend.

At probation review:

1. Selene prepares evidence.
2. Selene applies approved scorecard/policy rules.
3. Selene recommends pass, extend, or fail.
4. Selene routes protected outcome through HR, authority, simulation, and audit.

Selene may recommend or execute probation outcomes only through approved policy, simulation, authority, and audit. Selene must not casually terminate employees without protected HR/legal gates.

If company policy pre-authorizes deterministic probation rules, Selene may execute the outcome through approved simulation. If termination or non-confirmation is involved, protected HR/legal gate applies.

Example pass recommendation:

"Zara completed probation successfully. She completed 96% of assigned tasks, had no unapproved absences, and passed training. I recommend confirming her as permanent."

Example non-confirmation review:

"David did not meet probation criteria. He missed 5 accepted tasks, had 3 unexplained absences, and failed required safety training. I recommend non-confirmation review."

## 5. Resignation Flow

Employee says:

"Selene, I'm resigning as of June 30."

Selene must:

- verify employee identity,
- capture resignation date,
- check notice period,
- notify manager/HR where policy requires,
- prepare resignation record,
- identify open tasks,
- collect handover information,
- reallocate work,
- remove future roster assignments after final date,
- prepare access removal plan,
- prepare final pay estimate,
- calculate leave/benefit payout evidence,
- prepare tax/reporting handoff where applicable,
- audit.

Before final payment, Selene may require policy-authorized checks such as:

- handover completed,
- company property returned,
- open tasks summarized,
- contractual duties completed,
- final timesheet submitted.

Any handover-before-final-pay rule is policy and jurisdiction dependent. Selene must not unlawfully delay statutory final pay.

## 6. Manager Termination Flow

Manager says:

"Selene, terminate Tom effective Friday."

Selene must:

- verify manager authority,
- capture reason,
- classify termination reason category where policy requires,
- check jurisdiction and policy requirements,
- check notice requirements,
- prepare termination summary,
- prepare employee notice through PH1.WRITE,
- calculate final pay estimate,
- identify open tasks,
- collect handover or prepare fallback handover,
- remove future shifts,
- schedule access removal,
- prepare tax/reporting handoff,
- route to HR/management/legal approval if required,
- audit.

Termination categories may include:

- probation non-confirmation,
- resignation acceptance,
- redundancy,
- misconduct,
- performance-based termination,
- contract end,
- retirement,
- mutual separation,
- other policy-approved category.

Selene may inform the employee only after protected process authorizes it.

Employee notification must be respectful, direct, and policy-approved. PH1.WRITE owns the final employee-facing wording.

## 7. Work Reallocation On Exit

When an employee resigns or is terminated, Selene must:

- find all open tasks,
- find all scheduled work,
- find all contracts/milestones owned by the person,
- find unresolved messages/follow-ups,
- find assigned assets,
- find payroll/HR/access dependencies,
- prepare handover package,
- rank replacement candidates,
- push tasks to suitable people with context,
- notify originators,
- track transfer acceptance.

Replacement ranking should consider:

- capacity,
- skill match,
- certification,
- availability,
- overtime impact,
- location,
- priority,
- existing workload,
- acceptance tracking.

Example:

"Tom has 4 open tasks and 2 scheduled shifts. Sarah can take the stock reconciliation, and Michael can cover dispatch because he has capacity and the required certification. Do you want me to assign these?"

## 8. Records Retention

All employee changes must retain history:

- old address,
- old phone,
- old bank account,
- old salary,
- old position,
- old access,
- old leave balances,
- old manager,
- old contract terms,
- old tax details,
- old benefit elections.

Rule:

current record changes
-> historical record remains
-> audit links both

Sensitive history requires retention policy, access control, privacy classification, and audit.

## 9. What Must Not Happen

- no candidate outreach without approved channel/policy,
- no candidate screening using forbidden criteria,
- no probation termination without evidence and HR gate,
- no resignation final pay delay except where lawful and policy-approved,
- no termination message before protected approval,
- no old employee data deleted,
- no open tasks lost during offboarding,
- no access left active after employment ends,
- no GPT-5.5 final HR decision,
- no implementation from this document alone.

## 10. Final Architecture Sentence

Selene HR manages recruitment, candidate interaction, probation, resignation, termination, offboarding, rehire, handover, and employment lifecycle automation by using operational evidence, direct employee/candidate interaction, protected HR gates, and coordinated handoffs to Payroll, Access, Scheduler/Roster, Task/HWM, Finance, Reminder, and Delivery.
