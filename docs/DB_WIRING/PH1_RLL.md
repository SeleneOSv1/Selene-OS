# PH1_RLL DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.RLL
- layer: Offline Learning
- authority: Non-Authoritative
- role: Offline RL ladder ranking for governed artifact recommendations
- placement: OFFLINE_ONLY

## B) Ownership
- Tables owned: NONE (vNext design)
- Reads:
  - Structured, OS-supplied offline candidate artifacts from pattern/learning pipelines.
  - Offline metrics snapshots + reward signals (derived, bounded).
- Writes: NONE (no direct persistence in vNext)

## C) Hard Boundaries
- Non-authoritative and non-executing; recommendation output only.
- Must never grant authority, alter permissions, or bypass Access + Simulation ordering.
- Must never perform side effects, tool execution, or engine-to-engine direct calls.
- Offline-only constraint: never invoked in-turn; called only by offline pipeline orchestration.
- Governance constraint: output is recommendation only; activation requires governed artifact approval path.
- Tier-3 constraint: RL-driven artifact recommendations must require strict approval tier (Tier 3).

## D) Wiring
- Invoked_by: OS offline pipeline step: RL recommendation ranking.
- Inputs_from: PH1.PATTERN proposal outputs + offline reward/quality aggregates.
- Outputs_to: `ranked_rl_artifacts` bundle returned to Selene OS offline artifact queue.
- Invocation_condition: OFFLINE_ONLY
- Deterministic sequence:
  - `RLL_POLICY_RANK_OFFLINE` computes selected artifact + deterministic ordered recommendations.
  - `RLL_ARTIFACT_RECOMMEND` self-validates rank/selection/approval-tier integrity.
  - If `validation_status != OK`, Selene OS fails closed and does not forward recommendation bundle.
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Related Engine Boundaries
- Upstream: PH1.PATTERN supplies candidates via `PATTERN_MINE_OFFLINE -> PATTERN_PROPOSAL_EMIT`; PH1.RLL remains the offline ranker.
- Downstream: PH1.LEARN/PH1.PAE/PH1.PRUNE/PH1.CACHE/PH1.PREFETCH/PH1.CONTEXT may consume only governance-approved artifacts derived from PH1.RLL output.
- PH1.PAE may use RLL-derived artifacts only during `PAE_POLICY_SCORE_BUILD`; `PAE_ADAPTATION_HINT_EMIT` remains validation-only and cannot activate artifacts.

## F) Acceptance Tests
- AT-RLL-01: Selene OS can invoke `RLL_POLICY_RANK_OFFLINE` and output is schema-valid.
- AT-RLL-02: Output is bounded and deterministic ordering is preserved.
- AT-RLL-03: Non-offline usage fails closed (`offline_pipeline_only` required).
- AT-RLL-04: Recommendation validation drift fails closed before artifact queue handoff.
