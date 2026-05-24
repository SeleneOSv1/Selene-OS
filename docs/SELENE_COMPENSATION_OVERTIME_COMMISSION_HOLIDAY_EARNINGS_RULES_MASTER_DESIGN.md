# Selene Compensation, Overtime, Commission, Holiday + Earnings Rules Master Design

MASTER_DESIGN
NOT_RUNTIME_IMPLEMENTATION
PENDING_GRAND_ARCHITECTURE_RECONCILIATION

## 0. Authority and Scope

AGENTS.md controls execution.

This is a docs-only master design. No runtime code was changed. This document does not authorize Compensation implementation, Payroll implementation, tax/legal rule implementation, market-data provider integration, overtime execution, clock-out automation, migrations, packets, client edits, adapter edits, or old-path deletion.

The Payroll/HR repo-truth extraction remains the factual base. Current repo truth does not prove a Compensation business engine. Current `PH1.COMP` is deterministic computation, not compensation. This document defines future compensation and earnings-rule architecture pending Grand Architecture Reconciliation.

## 1. Purpose

Compensation provides the earning rules Payroll needs.

Selene must assist management in setting up compensation rules using:

- country,
- region,
- industry,
- company size,
- employee type,
- role,
- position,
- market research,
- company policy,
- legal/tax/payroll requirements.

Selene proposes numbers and rules. Management confirms or changes them. Payroll later uses approved deterministic rules.

## 2. Compensation Must Provide Payroll

Compensation must provide approved references for:

- base salary,
- hourly rate,
- overtime multiplier,
- weekend rate,
- public holiday rate,
- commission formula,
- bonus eligibility,
- allowances,
- benefits,
- approved salary changes,
- approved pay overrides.

Payroll uses those approved references to calculate pay. Compensation must not execute the payrun or move money.

## 3. Management Setup Flow

During company setup or position setup, Selene asks:

- What country/region does this company operate in?
- Which industry?
- What company size?
- What employee types do you use?
- Do hourly workers receive overtime?
- What overtime multiplier applies?
- What weekend rate applies?
- What public holiday rate applies?
- Do employees receive allowances?
- Do you use commissions?
- Do you pay bonuses?
- Do unused sick/annual leave accumulate or pay out?
- Do salary advances exist?

Selene may research/propose:

- market hourly rates,
- market salary bands,
- country tax/super/retirement rules,
- public holiday rules,
- industry patterns,
- standard overtime practices,
- benefit expectations.

Management then confirms or changes the proposal.

Example:

"I recommend $32/hour for this role based on region, industry, and company size. Overtime is commonly 1.5x after standard hours. Do you want to accept this or set a custom rate?"

No market, law, tax, contribution, or holiday claim may be final unless source-backed or owner-approved.

## 4. Updating Rates Over Time

Rates can change.

Flow:

1. authorized manager requests change,
2. Selene checks authority,
3. Selene shows current rate,
4. Selene asks effective date,
5. Selene shows affected employees,
6. Selene shows budget/payroll impact,
7. management confirms,
8. Compensation updates rule through approved simulation,
9. Payroll applies from effective date,
10. audit records old/new values and reason.

Example:

"Change weekend rate to 1.75x starting next month."

Selene must answer with impact preview:

"This affects 18 employees and increases estimated monthly payroll by $4,200. Do you want this to apply from June 1?"

## 5. Overtime Request And Approval Flow

Employee says:

"Selene, I need to work two extra hours."

Selene checks:

- current shift,
- task need,
- reason,
- overtime policy,
- budget,
- fatigue/rest limits,
- employee history,
- supervisor rules.

If policy allows automatic approval:

- overtime is approved,
- attendance expectation is updated,
- employee is reminded when overtime ends,
- payroll receives approved overtime evidence.

If supervisor approval is needed:

- Selene routes request,
- supervisor receives recommendation,
- employee sees simple status.

Selene should not expose bureaucracy to employee. Employee hears:

"Your overtime request is approved until 5 PM. I'll remind you before it ends."

Approved overtime feeds Payroll as evidence. Unapproved overtime creates exception, not automatic pay.

## 6. Fatigue, Rest, And Break Rule Boundary

Overtime approval must consider:

- maximum daily hours,
- minimum rest period,
- weekly hour cap,
- consecutive day cap,
- role-specific safety limits,
- paid break rules,
- unpaid break rules,
- missed break exceptions,
- meal penalty or compliance warning where applicable.

These rules must be source-backed or owner-approved. Selene must not invent labor-law limits.

## 7. Automatic Clock-Out / End-Of-Day Control

Selene must manage work-time boundaries where policy allows.

If employee's day ends:

- Selene reminds employee shift is ending.
- If policy allows auto clock-out, Selene clocks them out through Attendance.
- If employee remains on site waiting for a colleague, Selene records non-work presence if relevant.
- If employee wants to keep working, they must request overtime/extension.

Auto clock-out requires:

- policy allowed,
- employee notified,
- grace period where policy requires,
- manual override path,
- work-still-being-performed check,
- safety-role exception,
- site presence versus paid work distinction,
- audit.

Example:

"Your approved shift ended at 3 PM. I've clocked you out. If you need to keep working, request overtime approval."

## 8. Commission Boundary

Commissions can be complex and may belong partly to Sales.

Recommended owner split:

- Sales System owns sales events and sales truth.
- Compensation owns commission formula policy.
- Payroll pays approved commission.
- Finance validates money impact.

Selene should support:

- commission formula refs,
- sales event source refs,
- sales target refs,
- campaign refs,
- tiered rules,
- split commission,
- refund clawback,
- commission period,
- approval rules,
- dispute flow,
- paid/unpaid status.

Do not fully build commission inside Payroll alone.

## 9. Holiday And Special-Day Rules

Selene must identify each country/region's:

- public holidays,
- special work days,
- regional holidays,
- industry holidays,
- company holidays,
- holiday pay rules,
- holiday working restrictions.

Setup flow:

1. Selene researches holiday calendar.
2. Selene proposes annual holiday set.
3. Management confirms.
4. Calendar is locked for payroll/roster year.
5. Management may override dates/times.
6. Changes are audited.

Holiday model must handle:

- country,
- state/region,
- company calendar,
- observed holiday,
- substitute holiday,
- worked holiday,
- not-worked holiday,
- holiday during leave,
- holiday during weekend.

Example:

"I found 11 public holidays for Singapore this year. Please confirm these apply to your payroll and roster rules."

## 10. Tax / Super / Retirement / Benefit Rules

Selene must support country/region-specific:

- tax rates,
- PAYG/PAYE equivalents,
- superannuation,
- CPF,
- pension,
- social insurance,
- medical fund,
- housing fund,
- retirement contributions,
- employer/employee contributions,
- benefits.

Rules must be source-backed or owner-approved.

Selene can research and propose. Management, Payroll, Compliance, or Finance confirms. Payroll uses approved rules.

## 11. Bonus And Benefits Boundary

Bonus logic must define:

- bonus eligibility,
- bonus approval,
- performance source,
- discretionary versus formula bonus,
- payment date,
- forfeiture on resignation/termination where lawful,
- tax treatment.

Benefits logic must define:

- benefit enrollment,
- employee contribution,
- employer contribution,
- eligibility,
- deduction,
- termination handling,
- country-specific rules,
- provider handoff.

Compensation may define eligibility and package rules. Payroll applies approved pay effects. HR owns employment eligibility where applicable.

## 12. What Must Not Happen

- no GPT-5.5 invented pay law,
- no overtime paid without approved rule,
- no commission paid without Sales/Compensation evidence,
- no automatic overtime extension without policy,
- no holiday calendar used without confirmation,
- no rate change without effective date and audit,
- no automatic clock-out without company policy,
- no `PH1.COMP` treated as Compensation business truth,
- no implementation from this document alone.

## 13. Final Architecture Sentence

Selene Compensation owns the earning-rule brain: rates, overtime, weekend/public holiday pay, allowances, bonuses, commissions, benefits, tax/contribution assumptions, market-rate proposals, and management-confirmed compensation rules that feed Payroll accurately and automatically without becoming payroll execution or money movement.
