# PH1_PREFETCH DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.PREFETCH
- layer: Planning Assist
- authority: Non-Authoritative
- role: Read-only prefetch/cache warmer hinting
- placement: TURN_OPTIONAL

## B) Ownership
- Tables owned: NONE (vNext design)
- Reads:
  - Structured, OS-supplied intent/search context from upstream engine outputs.
  - Optional policy + privacy flags from Selene OS.
  - Optional search query hints and locale hints.
- Writes: NONE (no direct persistence in vNext)

## C) Hard Boundaries
- Non-authoritative and non-executing; advisory output only.
- Must never grant authority, alter permissions, or bypass Access + Simulation ordering.
- Must never perform side effects, tool execution, or engine-to-engine direct calls.
- Hard read-only rule: PH1.PREFETCH emits prefetch candidates only; PH1.E performs all tool calls.
- Hard dedupe/TTL rule: each candidate must carry deterministic idempotency dedupe key + bounded TTL for tool cache warming behavior.
- RLL consumption boundary: PH1.PREFETCH tuning hints may come only from governance-approved PH1.RLL artifacts.
- Cache boundary: PH1.PREFETCH may consume PH1.CACHE hints only as advisory ordering metadata; cache hints must not trigger autonomous tool execution.

## D) Wiring
- Invoked_by: OS step: after PH1.NLP intent classification / PH1.SEARCH hinting and before optional prefetch scheduling.
- Inputs_from: PH1.NLP intent_draft + PH1.SEARCH planning hints + optional PH1.CACHE route/skeleton hints + policy/privacy context.
- Outputs_to: `prefetch_candidates` bundle returned to Selene OS and forwarded to PH1.E scheduler/cache-warmer path.
- Invocation_condition: OPTIONAL(prefetch policy enabled)
- Deterministic sequence:
  - PREFETCH_PLAN_BUILD (derive candidate list with tool_kind/query/ttl/idempotency key/rank)
  - PREFETCH_PRIORITIZE (self-check candidate integrity and return deterministic prioritized ids)
  - If validation_status != OK, OS refuses/fails closed and does not forward bundle.
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Acceptance Tests
- AT-PREFETCH-01: Selene OS can invoke capability_id and output is schema-valid.
- AT-PREFETCH-02: Output is bounded and deterministic ordering is preserved.
- AT-PREFETCH-03: Candidate budget + privacy mode are enforced deterministically (read-only lanes only).
- AT-PREFETCH-04: Candidate/priority validation drift fails closed before downstream scheduling.
