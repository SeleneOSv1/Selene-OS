# Web Search Plan Canon

This file is the single source of truth for the Web Search Plan run sequence.

## Index
- Objective
- Architecture Flow
- Provider Strategy
- Packet Contract
- Determinism/Safety
- Requirement Catalog (Sections 0–38)
- Canonical Build Runs (Runs 1–30)
- Final Wiring Contract (Section 40)

## Objective
Deliver an evidence-bound, deterministic, fail-closed web research system with auditable contracts, reproducible outputs, and production release gates.

## Architecture Flow
1. `PH1.X` accepts and classifies turn intent.
2. Policy, access, and boundary gates are enforced.
3. Retrieval and transformation lanes build `EvidencePacket` and typed derivatives.
4. Synthesis and write layers render grounded outputs with citations.
5. Audit, replay, and quality gates enforce deterministic release behavior.

## Provider Strategy
- Provider usage is bounded by deterministic policy caps.
- Fallback ladders are explicit and ordered.
- Real-time and specialized modes are fail-closed when unconfigured or stale.
- No mode can bypass evidence authority.

## Packet Contract
- Canonical registries remain authoritative:
  - `PACKET_SCHEMAS.json`
  - `PACKET_REGISTRY.md`
  - `REASON_CODES.json`
  - `IDEMPOTENCY_KEYS.json`
  - `TURN_STATE_MACHINE.json`
- Schema changes require versioned compatibility and manifest updates.

## Determinism/Safety
- Stable ordering for all ranked, merged, and emitted artifacts.
- No hidden defaults, no random behavior, no silent fallbacks.
- Missing required evidence must fail closed with registered reason codes.
- Replay with fixed inputs must produce identical outputs.

## Requirement Catalog (Sections 0–38)
Catalog only; no run definitions in this section.

### Section 0
Foundational objectives and scope boundary.

### Section 1
Core contracts and canonical registries.

### Section 2
Proxy and outbound safety model.

### Section 3
URL retrieval and network fail-closed behavior.

### Section 4
Chunking, normalization, and hashing invariants.

### Section 5
Planning and retrieval pipeline constraints.

### Section 6
Evidence packet authority and provenance.

### Section 7
Web provider ladder behavior.

### Section 8
News provider ladder behavior.

### Section 9
Top-K selection and opening policy.

### Section 10
Cross-provider fallback discipline.

### Section 11
Learning boundaries and governance constraints.

### Section 12
Grounded synthesis contract.

### Section 13
Write and TTS output contract.

### Section 14
Observability and debug diagnostics requirements.

### Section 15
Performance and cost policy requirements.

### Section 16
Caching and deterministic parallelism requirements.

### Section 17
Replay harness and quality regression requirements.

### Section 18
Failure-signature learning and rollback governance.

### Section 19
Structured connector requirements.

### Section 20
Document parsing and filing-aware extraction requirements.

### Section 21
Numeric and consensus analytics requirements.

### Section 22
Competitive comparison requirements.

### Section 23
Real-time TTL and stale refusal requirements.

### Section 24
Regulatory and jurisdiction filter requirements.

### Section 25
Trust, spam, and official-source model requirements.

### Section 26
Multi-hop planning and cycle safety requirements.

### Section 27
Temporal as-of and deterministic diff requirements.

### Section 28
Risk scoring and non-advice guardrail requirements.

### Section 29
Internal/external merge boundary requirements.

### Section 30
Production lock and release-gate requirements.

### Section 31
Go-live release operations requirements.

### Section 32
Continuous evaluation requirements.

### Section 33
Capability parity enhancement requirements.

### Section 34
Gap-closer safety and transparency requirements.

### Section 35
Enterprise integration lock and cross-mode consistency requirements.

### Section 36
Document canon enforcement requirements.

### Section 37
Release evidence and traceability requirements.

### Section 38
System-wide acceptance and governance requirements.

## Section 39 — Canonical End-to-End Build Program

## Canonical Build Runs (Runs 1-30)

#### Run 1 — Foundation Contracts
Establish canonical packets, reason-code registry, idempotency registry, and turn state machine checks.

#### Run 2 — Proxy Universal Layer
Implement deterministic proxy support, redaction, and fail-closed proxy validation.

#### Run 3 — URL Fetch Core
Implement safe URL retrieval with caps, timeout discipline, and deterministic provenance.

#### Run 4 — Chunk/Hash Core
Implement deterministic chunking, normalization, and content hash generation.

#### Run 5 — Planning Spine Initialization
Establish deterministic planning boundaries and packet handoff discipline.

#### Run 6 — Retrieval Lane Hardening
Harden retrieval plumbing, URL-open control points, and fail-closed pathing.

#### Run 7 — Web Provider Ladder
Implement deterministic web ladder ordering and fallback behavior.

#### Run 8 — Planning/Selection Expansion
Extend deterministic candidate selection and evidence preparation logic.

#### Run 9 — Search Top-K Pipeline
Implement deterministic top-k selection/open behavior with bounded retries and fallback handling.

#### Run 10 — News Provider Ladder
Implement deterministic news retrieval ladder with recency and provenance constraints.

#### Run 11 — Learning Layer (Governed)
Implement bounded failure signatures, session adaptation, proposal artifacts, promotion gate, and rollback safeguards.

#### Run 12 — Grounded Synthesis Engine
Implement strict claim-to-citation synthesis with conflict surfacing and insufficient-evidence refusal.

#### Run 13 — Write/TTS Contract
Implement deterministic write formatting, citation rendering, localization guard, and voice parity from formatted text.

#### Run 14 — Observability Debug Packet
Implement deterministic debug packet generation, redaction, error taxonomy mapping, and state trace capture.

#### Run 15 — Perf/Cost/Importance Tiers
Implement deterministic budgets, timeouts, degradation order, and audit fields.

#### Run 16 — Cache + Parallel Retrieval
Implement two-tier cache and deterministic parallel scheduling/merge with cache safety guarantees.

#### Run 17 — Replay + Quality Gates
Implement deterministic replay harness, golden snapshots, and strict regression thresholds.

#### Run 18 — Learning Layer Governance Extension
Harden non-authoritative learning controls, explicit approval promotion, and deterministic rollback drills.

#### Run 19 — Structured Connectors
Implement deterministic structured extraction framework, schema validation, and adapter routing.

#### Run 20 — Document Parsing
Implement deterministic PDF/text/table/filing-aware parsing and typed row output.

#### Run 21 — Numeric & Consensus Analytics
Implement deterministic evidence-bound aggregate/consensus computation output.

#### Run 22 — Competitive Intelligence Mode
Implement deterministic entity/feature/pricing normalization and evidence-bound comparison outputs.

#### Run 23 — Real-Time API Mode
Implement deterministic domain routing, TTL policy, freshness scoring, and stale-data refusal.

#### Run 24 — Regulatory/Jurisdiction Mode
Implement deterministic jurisdiction resolution, trust-tier filtering, and compliance confidence gates.

#### Run 25 — Trust/Spam/Official Source Model
Implement deterministic source trust scoring, spam penalties, and explainable trust factors.

#### Run 26 — Deep Multi-Hop Planner
Implement deterministic multi-hop planning with hard budgets, cycle detection, and hop proof chain.

#### Run 27 — Temporal Comparison Mode
Implement deterministic as-of windows, timeline extraction, and typed diff outputs.

#### Run 28 — Risk Scoring Mode
Implement deterministic factorized risk scoring, confidence coverage, and non-advice guardrails.

#### Run 29 — Internal+External Merge Mode
Implement deterministic merge packet, evidence-supremacy boundary enforcement, delta/conflict reporting.

#### Run 30 — Production Lock + Final Acceptance
Implement full release lock matrix, trace matrix gate, SLO lock, and release evidence pack generation.

## Final Wiring Contract (Section 40)
- Canonical execution order is fixed: classification -> access/policy gates -> retrieval/transforms -> synthesis/write -> audit.
- No alternate authority path may bypass canonical boundaries.
- Internal context is advisory only; external evidence remains truth authority.
- All release decisions are no-release-on-red and must be reproducible.
