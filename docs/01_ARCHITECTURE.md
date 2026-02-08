# Selene OS Architecture

## Section A: Purpose
Selene OS is an orchestration-first operating runtime that enforces deterministic execution, safety gates, and auditable outcomes across voice, reasoning, tools, and domain workflows.

## Section B: Kernel vs Constitution (separation)
The kernel defines executable runtime contracts, schemas, and enforcement boundaries. The constitution defines behavioral laws, safety posture, and product principles. Kernel is code-enforced mechanics; constitution is system-wide governance and operating doctrine.

## Section C: Global Gate Order (Identity → STT → NLP → Confirm → Access → Simulation → Domain → Persist/Audit)
All actionable work follows one global order: Identity, then STT, then NLP understanding, then user confirmation (when required), then access/authority, then simulation eligibility, then domain execution, and finally persistence plus audit logging.

## Section D: Engine Orchestration Law (WorkOrder, Blueprint, Simulation)
Selene OS orchestrates by binding each task to a WorkOrder, selecting a Blueprint for deterministic step order, and allowing execution only through declared Simulations for side-effect safety.

## Section E: Engine Call Rule (“Engines never call engines”)
Engines are pure workers. Engines return structured results to Selene OS. Only Selene OS may orchestrate cross-engine flow.

## Section F: No Simulation → No Execution
Any operation with side effects is blocked unless an active simulation exists, all required gates pass, and required confirmations are complete.

## Section G: MVP pipeline summary (PH1.K → PH1.VOICE.ID → PH1.W → PH1.C → PH1.NLP → PH1.X → PH1.D/PH1.E → PH1.TTS → PH1.L + PH1.M/F/J/EXPLAIN)
MVP runtime flow starts with PH1.K voice substrate, PH1.VOICE.ID identity, PH1.W wake, PH1.C transcript quality, PH1.NLP intent draft, PH1.X orchestration, PH1.D/PH1.E constrained reasoning/tool routing, PH1.TTS speech output, and lifecycle/memory/persistence/audit/explain handling through PH1.L and PH1.M/F/J/EXPLAIN.
