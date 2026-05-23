# Selene PH1.REM — Reminder Journey Intelligence + Human Follow-Up Master Design

DOCUMENT STATUS:
MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority and Scope

AGENTS.md controls execution.

This is docs-only master design.

No runtime code was changed.

This document does not authorize implementation.

The PH1.REM repo-truth extraction remains the factual base:

- `docs/SELENE_PH1REM_REMINDER_ENGINE_REPO_TRUTH_FUNCTIONALITY_EXTRACTION_MASTER_DESIGN.md`

This document defines future reminder journey architecture pending Grand Architecture Reconciliation.

Future implementation requires repo-truth activation, approved file scope, tests, backend evidence, PH1.D provider-off/fake-provider proof, PH1.WRITE validation proof, BCAST/DELIVERY proof where delivery occurs, audit proof, and JD live acceptance.

## 1. Executive Target

PH1.REM is Selene's deterministic timing truth engine.

Selene Reminder Journey Intelligence is the human layer around it.

The target is:

- understand messy human reminder requests;
- distinguish one-shot reminders from follow-up reminders;
- handle birthdays, anniversaries, special days, regional holidays, appointments, onboarding follow-ups, link follow-ups, tasks, and other people reminders;
- use GPT-5.5 / PH1.D / PH1.N / PH1.X / PH1.WRITE for natural human behavior;
- schedule deterministic timing through PH1.REM;
- deliver through PH1.BCAST / PH1.DELIVERY where needed;
- use status/troubleshooting assistants;
- preserve audit;
- avoid mutating task/scheduler/roster/access/protected state from PH1.REM alone.

The user-facing standard is:

Selene should understand reminders the way humans ask for them.

The system standard is:

Selene may understand and phrase reminder journeys probabilistically, but timing, delivery, access, authority, audit, and source truth must stay deterministic and owner-scoped.

## 2. Current Repo-Truth Foundation

The PH1.REM repo-truth extraction establishes:

- PH1.REM owns schedule/update/cancel/snooze/retry/follow-up/escalate/complete/fail timing mechanics.
- PH1.REM currently uses OS + PH1.F storage; standalone `crates/selene_engines/src/ph1rem.rs` was not found.
- PH1.REM currently has limited time parsing.
- PH1.REM has daily/weekly recurrence support but not full rich recurrence.
- PH1.REM does not own external message delivery.
- PH1.REM does not own final wording.
- PH1.REM does not own tasks/scheduler/roster/onboarding state.
- PH1.REM needs a human journey upgrade.

This document does not duplicate the full extraction. It defines the future journey intelligence that should sit above current PH1.REM mechanics.

## 3. Master Owner Split

PH1.REM owns:

- timing truth;
- reminder state;
- occurrence state;
- snooze;
- follow-up;
- retry timing;
- recurrence timing;
- completion/failure state;
- reminder timing audit evidence.

PH1.REM does not own:

- final wording;
- external delivery;
- contact permission;
- birthday/contact truth;
- holiday/calendar truth;
- appointment truth;
- task truth;
- onboarding state;
- link state;
- roster/scheduler state;
- access;
- authority;
- protected execution.

Truth owner examples:

- Birthday / anniversary truth: PH1.M / contact/person profile owner where consent allows.
- Regional holiday truth: Calendar/Holiday owner and PH1.E/Search where freshness/public verification is needed.
- Appointment truth: Calendar/Appointment owner.
- Task truth: Task owner.
- Onboarding state: PH1.ONB.
- Link state: PH1.LINK.
- Delivery state: PH1.BCAST / PH1.DELIVERY.
- Final human wording: PH1.WRITE.
- OpenAI proposal gateway: PH1.D.

Final rule:

PH1.REM owns when.

The source owner owns what.

PH1.WRITE owns how Selene says it.

## 4. Probabilistic + Deterministic Reminder Model

Reminder user experience should feel human and intelligent.

Use probabilistic stack for understanding and communication:

- PH1.D / GPT-5.5 semantic proposal;
- PH1.N extraction of time/action/recipient/recurrence/ambiguity;
- PH1.X route/risk validation;
- PH1.WRITE final wording;
- Selene Emotional Intelligence where helpful;
- Universal Language for multilingual reminders.

Use deterministic owners for:

- exact scheduled time;
- recurrence;
- completion;
- snooze;
- delivery state;
- status;
- audit;
- permissions;
- task/scheduler/roster/onboarding/link mutation.

Required law:

GPT-5.5 helps interpret and speak.

PH1.REM schedules and tracks timing.

PH1.X validates.

PH1.WRITE finalizes.

Stack owners perform their own work.

## 5. Reminder Intent Classes

| Class | Example | Likely Truth Owner | PH1.REM Timing | BCAST/DELIVERY Needed | Confirmation | Follow-Up | Access / Authority / Simulation |
| --- | --- | --- | --- | --- | --- | --- | --- |
| One-shot reminder | "Remind me tomorrow at 10 about the dentist." | User reminder request; appointment owner if tied to appointment | Yes | Usually in-app/push; external optional | Lightweight if clear | Usually none | Simulation for scheduling; access user-scoped. |
| Occasion reminder | "It's Tom's birthday today." | PH1.M/contact profile where consent allows | Yes for future occasion | Optional | Clarify if source/date unknown | Usually none | Memory/contact access required. |
| Generated occasion message reminder | "Remind me to wish Tom happy birthday." | PH1.M/contact profile plus PH1.WRITE for wording | Yes | Maybe if drafting/sending to Tom | Confirmation before send | Usually none | Delivery permission if external. |
| Appointment reminder | "Remind me 30 minutes before my dentist appointment." | Calendar/Appointment owner | Yes for offset timing | Usually local/push | Lightweight if appointment resolved | Optional | Calendar access required. |
| Action-until-complete reminder | "Keep reminding me to submit payroll until it's done." | Task/Payroll owner for completion truth | Yes for repeat timing | Maybe | Required | Repeat until complete/expired | Protected/private risk; simulation and authority may apply. |
| Other-person reminder | "Remind my daughter she needs to finish homework before 6 PM." | Sender/recipient/contact owners | Yes | Yes if sent externally | Required | Optional | Contact/delivery permission required. |
| Onboarding postpone/resume reminder | "Not now, remind me tomorrow to finish onboarding." | PH1.ONB | Yes | Maybe | Usually lightweight after valid onboarding session | Resume exact step | ONB access/session scope required. |
| Link follow-up reminder | "Remind Tom to finish the invite link tomorrow." | PH1.LINK | Yes | Yes if reminding Tom | Required | Optional/ack | Link status access required. |
| Task/scheduler/roster reminder | "Remind Sarah before her shift starts." | Task/Scheduler/Roster owner | Yes | Maybe | Required when recipient is another person | Depends on source owner | Schedule/roster access may be required. |
| Recurring reminder | "Remind me every Monday." | User reminder request; source owner if tied to business object | Yes | Optional | Required for recurrence | Recurs until canceled/expired | Simulation required; risk depends content. |
| Regional holiday / special day reminder | "Remind me before Singapore National Day." | Calendar/Holiday owner; PH1.E/Search for current public verification | Yes | Optional | Clarify region/date if ambiguous | Usually none | Freshness/source proof may be required. |
| Emergency/critical reminder | "Keep reminding the manager until the safety check is acknowledged." | Safety/compliance/task owner | Yes | Yes | Serious confirmation | Escalating | Explicit simulation/policy/authority may be required. |
| Unknown/unsupported reminder type | "Remind someone sometime somehow." | Unknown until clarified | No schedule until resolved | Unknown | Clarification required | Unknown | No execution until resolved. |

## 6. Birthdays, Anniversaries, Special Days, And Regional Holidays

PH1.REM should support timing for:

- birthdays;
- anniversaries;
- personal milestones;
- special days;
- holidays relevant to region/country;
- company events;
- cultural/public holidays where allowed;
- business deadlines.

But PH1.REM does not own the date truth.

Owner map:

- personal birthdays/anniversaries: PH1.M / contact profile / relationship memory owner, consent-scoped;
- company special days: company/tenant/workspace owner;
- regional holidays: Calendar/Holiday owner and PH1.E/Search where current verification is required;
- payroll/HR holidays: Payroll/HR/calendar owner;
- appointment reminders: Calendar/Appointment owner.

Example behavior:

User-visible reminder:

`It's Tom's birthday today. Want me to help write something warm?`

For birthday/special-day reminders, Selene should not repeat the exact same phrase every year.

PH1.D/GPT-5.5 may propose fresh, human wording.

PH1.WRITE validates final wording.

PH1.REM owns timing only.

## 7. Other-Person Reminders

Selene must support sender-created reminders for another person.

Examples:

- "Remind my daughter she needs to finish her homework before 6 PM."
- "Remind my wife that I'm still her husband."
- "Remind Tom to complete onboarding tomorrow."

This creates a sender-initiated recipient reminder.

Rules:

- sender must be allowed to contact/remind recipient;
- recipient contact/channel must be valid;
- PH1.BCAST/DELIVERY owns outbound delivery;
- PH1.REM owns timing;
- PH1.WRITE owns reminder wording;
- recipient privacy/consent rules apply;
- the reminder should feel to the recipient like a normal Selene reminder where allowed;
- audit must preserve that the reminder was initiated by someone else;
- if recipient accepts/adopts it, future follow-up may become recipient-owned where policy allows.

Example:

Sender: "Remind my daughter she needs to finish homework before 6 PM."

Selene resolves:

- recipient = daughter;
- action = finish homework;
- deadline = before 6 PM;
- delivery channel = approved contact method;
- follow-up policy = maybe check/confirm if sender requested.

Selene asks confirmation:

`Just confirming: should I remind your daughter before 6 PM to finish her homework because you're going out for dinner?`

## 8. Follow-Up Policy Model

Selene must separate reminders by follow-up behavior.

Types:

1. No-follow-up reminder
   - Examples: birthday greeting reminder, special-day reminder, simple one-shot reminder.

2. Optional follow-up reminder
   - Example: "Remind me about my dentist appointment 30 minutes before."
   - Default one reminder; user may ask for more.

3. Confirmation-required reminder
   - Example: "Remind me to leave for the dentist and keep reminding me until I confirm I'm leaving."

4. Action-until-complete reminder
   - Example: "Keep reminding me to submit payroll until it's done."

5. Recipient-acknowledgement reminder
   - Example: "Remind Sarah and let me know when she confirms."

6. Escalating reminder
   - Example: "Keep following up until the compliance document is uploaded."

7. Postponed/resume reminder
   - Example: "Not now, remind me tomorrow to finish onboarding."

Each reminder must have a follow-up policy:

- none;
- ask once;
- repeat until acknowledged;
- repeat until completed;
- escalate after N failures;
- expire at deadline;
- notify requester;
- hand off to canonical owner.

PH1.REM tracks timing/follow-up.

PH1.BCAST/DELIVERY tracks external delivery.

Source owner tracks completion truth.

## 9. Confirmation And Clarification Policy

Not every reminder needs heavy confirmation.

Confirmation levels:

1. Lightweight acknowledgement

For clear, low-risk personal reminders:

`Done - I'll remind you tomorrow at 10 AM about your dentist appointment.`

2. Clarification required

For vague time, recipient, or action:

`Sure - later today, tonight, or tomorrow?`

3. Confirmation required

For:

- reminders to someone else;
- recurring reminders;
- external delivery;
- private/sensitive content;
- work/business reminders;
- onboarding/link follow-up;
- action-until-complete reminders;
- medical/high-risk reminders;
- escalation reminders.

Example:

`Just confirming: should I remind Sarah every weekday at 9 AM until she uploads the compliance form?`

4. Serious confirmation

For:

- medical/health reminders;
- payroll/HR/finance reminders;
- emergency/critical reminders;
- broad audience reminder flows;
- reminders that may trigger escalation.

PH1.WRITE must produce human confirmation language.

PH1.D/GPT-5.5 may draft.

PH1.X validates whether confirmation is required.

## 10. Natural Language Understanding + Time Intelligence

Users may say:

- "remind me later";
- "after lunch";
- "next Friday morning";
- "end of shift";
- "before dinner";
- "when Tom gets back";
- "every second Tuesday";
- "monthly on the 15th";
- "before Singapore public holidays";
- "half an hour before the appointment."

Required future flow:

User phrase
-> PH1.D/GPT-5.5 proposes meaning
-> PH1.N extracts candidates
-> PH1.X validates ambiguity
-> PH1.WRITE asks clarification when needed
-> PH1.REM schedules only after time is valid.

Rich time interpretation must include:

- exact date/time;
- relative time;
- fuzzy time;
- time windows;
- timezone;
- locale;
- daylight saving;
- holidays;
- work shifts;
- roster/schedule context;
- appointment-relative offsets;
- recurrence.

Ambiguous times require clarification:

`Do you mean next Friday morning around 9 AM?`

Do not silently guess for important reminders.

## 11. Regional Holiday / Calendar Intelligence

Selene must support region-aware holiday reminders.

Examples:

- "Remind me before public holidays in Singapore."
- "Remind me before Australian payroll holidays."
- "Tell me before school holidays start."

Rules:

- PH1.REM owns reminder timing.
- Calendar/Holiday owner owns holiday definitions.
- PH1.E/Search may verify current public holiday data when needed.
- Payroll/HR/Scheduler owners may own business holiday impact.
- PH1.WRITE explains.
- User location/region must be scoped and confirmed when ambiguous.
- Holidays can differ by country/state/region/industry/company policy.

## 12. Reminder Delivery And Channel Handoff

Reminder visibility/audibility must be governed.

Possible delivery channels:

- in-app;
- push;
- SMS;
- email;
- voice/TTS;
- phone app;
- future approved channels.

Rules:

- PH1.REM does not become delivery provider.
- PH1.BCAST/DELIVERY owns outbound communication where needed.
- PH1.TTS owns spoken reminder output.
- PH1.WRITE owns wording.
- Desktop/iPhone render only.
- Adapter transports only.
- provider-off/fake-provider/live-provider proof required before live claims.
- delivery failures must be visible to status assistant.

## 13. Onboarding Postpone / Resume Reminders

Selene must support onboarding postponement.

Example:

Receiver says:

`Not now, remind me tomorrow.`

Flow:

- PH1.N interprets requested time;
- PH1.X validates reminder route;
- PH1.REM schedules;
- PH1.BCAST/DELIVERY delivers follow-up if needed;
- PH1.ONB resumes exact remaining step.

Rule:

Never restart onboarding if valid progress exists.

Never ask again for completed valid fields.

## 14. Link Follow-Up Reminders

Selene must support link follow-up timing.

Examples:

- "Remind Tom to finish the invite link tomorrow."
- "Tell me if Sarah still hasn't opened the link by Friday."

Owner split:

- PH1.LINK owns link status;
- PH1.REM owns follow-up timing;
- PH1.BCAST/DELIVERY owns reminder delivery;
- PH1.WRITE explains;
- Access/Governance scopes who may see link status.

## 15. Task / Scheduler / Roster Boundary

PH1.REM must not mutate tasks, schedules, rosters, or workloads by itself.

Examples:

`Remind Sarah before her shift.`

This means:

- PH1.REM timing may depend on Scheduler/Roster truth.
- Scheduler/Roster owner owns shift truth.

`Remind me to submit the task until done.`

This means:

- Task owner owns task completion truth.
- PH1.REM owns follow-up timing.

Future Scheduler/Roster integration must use canonical owners and simulations.

## 16. Status Assistant

Selene must answer:

- What reminders do I have?
- What is due next?
- Did the reminder fire?
- Did Tom receive it?
- Did Sarah acknowledge it?
- Is it recurring?
- When will you remind me again?
- Why did the reminder fail?
- Can you cancel or change it?
- Did onboarding resume?
- Did the link get opened?

Owner map:

- PH1.REM = timing state;
- PH1.BCAST/DELIVERY = delivery state;
- PH1.LINK = link state;
- PH1.ONB = onboarding state;
- Task/Scheduler/Roster = business state;
- PH1.WRITE = explanation.

Status answers must be access-scoped.

## 17. Troubleshooting Assistant

Selene must troubleshoot:

- ambiguous time;
- wrong time;
- timezone mismatch;
- reminder not delivered;
- recipient did not get reminder;
- recipient did not acknowledge;
- recurrence wrong;
- reminder fired but task not done;
- onboarding reminder did not resume;
- link follow-up failed;
- delivery provider unavailable;
- quiet hours blocked reminder;
- user wants to change/cancel/snooze.

PH1.WRITE must explain in human language.

Example:

`I couldn't schedule that because "after lunch" is ambiguous. Do you want 1 PM?`

## 18. Privacy / Safety / Consent

Reminder content can be sensitive.

Rules:

- health/medical reminders need safety policy;
- payroll/HR/finance reminders need privacy/access checks;
- reminders to someone else require contact/recipient permission;
- repeated reminders must not become harassment;
- recipient opt-out/stop preferences must be honored where applicable;
- quiet hours and do-not-disturb policies must be respected unless emergency policy allows override;
- emergency/critical reminders require explicit simulation/policy;
- OpenAI may not see private reminder content unless data-egress policy allows it.

## 19. Required Future Packets

Logical future packets:

- ReminderJourneyRequestPacket
- ReminderIntentCandidatePacket
- ReminderTimeCandidatePacket
- ReminderRecipientCandidatePacket
- ReminderRecurrenceCandidatePacket
- ReminderAmbiguityPacket
- ReminderConfirmationPacket
- ReminderFollowupPolicyPacket
- ReminderOccasionPacket
- ReminderHolidayContextPacket
- ReminderOtherPersonRequestPacket
- ReminderDeliveryHandoffPacket
- ReminderStatusSummaryPacket
- ReminderTroubleshootingPacket
- ReminderCompletionCheckPacket
- ReminderSourceOwnerPacket
- ReminderAuditEvidencePacket

Codex must later map these to repo equivalents.

Do not claim they currently exist.

## 20. Example End-To-End Flows

Example A - Birthday:

User: "Remind me when it's Tom's birthday."

Flow:

PH1.M/contact owner stores birthday where permitted
-> PH1.REM schedules
-> PH1.WRITE says: "It's Tom's birthday today. Want me to help write something warm?"

Example B - Dentist:

User: "Remind me 30 minutes before my dentist appointment."

Flow:

Appointment owner provides appointment time
-> PH1.REM schedules offset reminder
-> PH1.WRITE acknowledges
-> no follow-up unless requested.

Example C - Keep reminding until done:

User: "Keep reminding me to submit payroll until it's done."

Flow:

PH1.X marks business/private risk
-> Payroll/task owner owns completion truth
-> PH1.REM schedules repeated follow-up
-> stop when source owner marks done.

Example D - Remind daughter:

User: "Remind my daughter she needs to finish homework before 6 PM."

Flow:

recipient resolved
-> sender permission/contact check
-> PH1.WRITE drafts human reminder
-> confirmation
-> PH1.REM schedules
-> PH1.BCAST/DELIVERY sends to daughter.

Example E - Wife reminder:

User: "Remind my wife that I'm still her husband."

Flow:

Selene treats as personal outreach reminder
-> checks recipient/contact/delivery permission
-> PH1.WRITE may use warm/playful wording if appropriate
-> confirmation
-> sends via delivery owner if allowed.

Example F - Onboarding postpone:

User: "Not now, remind me tomorrow."

Flow:

PH1.REM schedules
-> later reminder resumes exact PH1.ONB step.

Example G - Ambiguous time:

User: "Remind me after lunch."

Flow:

PH1.D/PH1.N propose fuzzy time
-> PH1.X marks ambiguous
-> PH1.WRITE asks: "Do you mean today around 1 PM?"

Example H - Regional holiday:

User: "Remind me before Singapore public holidays."

Flow:

region confirmed
-> holiday owner/PH1.E verifies dates
-> PH1.REM schedules
-> PH1.WRITE explains.

## 21. What Must Not Happen

Codex must not allow:

- no rebuilding PH1.REM from scratch;
- no duplicate reminder engine;
- no GPT-5.5 direct scheduling;
- no PH1.REM owning message wording;
- no PH1.REM owning delivery provider;
- no PH1.REM mutating task/scheduler/roster state;
- no PH1.REM owning birthday/contact/holiday truth;
- no reminder to someone else without recipient/contact/delivery permission;
- no repeated reminders that become harassment;
- no sensitive reminder content without privacy/access checks;
- no medical reminder product behavior without safety policy;
- no broad recurrence or holiday claims without source-owner proof;
- no live provider claim without provider proof;
- no Desktop/iPhone authority;
- no Adapter authority;
- no implementation from this document alone.

## 22. Required Upgrade List

Exact upgrade points:

1. GPT-5.5 / PH1.D natural-language reminder understanding
2. PH1.N extraction of time, recipient, action, recurrence, ambiguity
3. PH1.X route/risk validation for reminders
4. PH1.WRITE reminder wording and clarification
5. rich time interpretation: "after lunch", "next Friday morning", "end of shift"
6. timezone / locale / daylight-saving policy
7. monthly/custom recurrence
8. onboarding postpone/resume reminders
9. reminder status assistant
10. live provider delivery proof
11. quiet-hours policy
12. task/scheduler/roster boundary map
13. PH1.J audit proof
14. SQL persistence/migration proof
15. Desktop/iPhone render-only proof
16. Adapter transport-only proof
17. JD live reminder acceptance

Suggested upgrade slices:

1. Reminder Journey Intelligence layer
2. PH1.D / GPT-5.5 reminder understanding
3. PH1.N time/action/recipient extraction
4. PH1.X reminder route validation
5. PH1.WRITE reminder clarification and status wording
6. richer time parser with clarification
7. timezone / locale / DST policy
8. recurrence expansion
9. onboarding postpone/resume integration
10. link follow-up reminders
11. delivery/channel handoff to BCAST/DELIVERY
12. quiet-hours / interruption policy
13. reminder status assistant
14. reminder troubleshooting assistant
15. task/scheduler/roster boundary map
16. PH1.J audit evidence
17. SQL persistence plan
18. JD live reminder proof

## 23. Recommended Future Build Slices

1. Reminder Journey Intelligence Activation Pack
2. PH1.D / GPT-5.5 Reminder Understanding Shell
3. PH1.N Time / Action / Recipient / Recurrence Extraction
4. PH1.X Reminder Route / Risk Validation
5. PH1.WRITE Reminder Wording Boundary
6. Rich Time + Ambiguity Clarification
7. Timezone / Locale / DST Policy
8. Recurrence Expansion
9. Birthdays / Anniversaries / Special Days / Holiday Reminder Support
10. Other-Person Reminder Flow
11. Follow-Up Policy Engine
12. Onboarding Postpone / Resume Reminder Proof
13. Link Follow-Up Reminder Proof
14. Delivery Channel Handoff to BCAST/DELIVERY
15. Quiet Hours / Interruption Policy
16. Reminder Status Assistant
17. Reminder Troubleshooting Assistant
18. Task / Scheduler / Roster Boundary Map
19. PH1.J Audit Evidence Pack
20. SQL Persistence / Migration Plan
21. Desktop/iPhone Render-Only Reminder Proof
22. Adapter Transport-Only Reminder Proof
23. JD Live Reminder Acceptance Pack

## 24. Grand Architecture Reconciliation Note

This document must later be reconciled into:

- PH1.REM extraction;
- PH1.D Proposal Gateway;
- PH1.N Meaning Unravelling;
- PH1.X Request Decision Lattice;
- PH1.WRITE Human Presentation;
- PH1.M Human Memory;
- PH1.BCAST / PH1.DELIVERY;
- PH1.ONB Onboarding Journey;
- PH1.LINK Link Journey;
- Broadcast Advanced Delivery Modes;
- Task/Scheduler/Roster future stacks;
- Calendar/Holiday owners;
- Provider Governance;
- Identity + Access + Authority;
- Desktop/iPhone render-only proof;
- Adapter transport-only proof;
- Old Compatibility Path Retirement.

## 25. Final Architecture Sentence

PH1.REM is Selene's reminder timing truth engine; Reminder Journey Intelligence lets Selene understand human reminder requests, clarify fuzzy time, separate one-shot reminders from follow-up journeys, support birthdays, anniversaries, holidays, appointments, onboarding, links, tasks, and reminders for other people, phrase reminders naturally through PH1.WRITE and GPT-5.5, and route delivery/status/troubleshooting through the correct owners without letting reminders mutate tasks, onboarding, access, schedules, rosters, contacts, holidays, or protected business state.
