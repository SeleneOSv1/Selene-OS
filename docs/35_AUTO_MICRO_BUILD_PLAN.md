# AUTO_MICRO — Reporting → Action → Promotion (Build Plan)

## 0. Design Status
- This document is design-only.
- No schema migration, no runtime wiring, no simulation activation is performed by this plan document.
- AUTO_MICRO is defined as a simulation class and policy flow, not a new engine.

## 1. Goal
Build a governed closed loop where deterministic runtime signals become deterministic scorecards, scorecards produce deterministic proposals/actions, and only approved actions reach promotion paths with rollback.

Flow:
`Signal Capture -> Scorecard -> Report -> Action/Proposal/Blocked -> Promotion -> Rollback/Keep`

## 2. Scope
In scope:
- Signal capture unification from FEEDBACK, LEARN, and outcome-utilization across STT/TTS/NLP/LLM/OCR/Connectors.
- Provider/domain scorecards with deterministic formulas and thresholds.
- Promotion ladder policy (`Shadow -> Assist -> Lead`) with deterministic promote/demote/rollback.
- AUTO_MICRO simulation class for governed actions and proofs.
- No-dead-reporting enforcement.
- Governance boundaries with builder-only activation for non-LOW risk.

Out of scope:
- New engine identity.
- Any irreversible external side effect outside existing simulation-governed commit paths.
- Any permission model bypass (must use per-user PH2 access instance).

## 2.1 Non-Goals / Forbidden Scope (Hard Stop)
AUTO_MICRO must never touch or mutate:
- Money movement or payment execution paths.
- Permission grants/denials, approval decisions, or authority routing.
- External irreversible sends/commits outside already-allowed LOW-risk policy commits.
- Schema changes, migrations, or table ownership changes.
- Onboarding lifecycle state or onboarding identity gates.
- Access instance lifecycle, access overlays, access board votes, or AP schema flows.
- Payroll/HR business state or payroll/HR commit simulations.

If a proposal/action intersects any item above, AUTO_MICRO must fail closed and route to builder/governance review only.

## 3. Baseline Inputs (Existing Surfaces)
- FEEDBACK signal bundles (non-authoritative).
- LEARN artifact packages and self-heal cards (non-authoritative promotion metadata).
- Outcome-utilization append-only rows.
- Existing builder/governance rollout and rollback discipline.
- Existing simulation activation guard + access gate patterns.

## 4. Unified Signal Capture Plan

### 4.1 Signal Planes
Define one normalized signal envelope for AUTO_MICRO reporting:
- `signal_plane`: `FEEDBACK | LEARN | OUTCOME_UTILIZATION`
- `modality`: `STT | TTS | NLP | LLM | OCR | CONNECTOR`
- `provider_id`: normalized token (or `NONE`)
- `domain_key`: deterministic domain bucket (intent/route/tool lane)
- `tenant_id`, `correlation_id`, `turn_id`, `reason_code`, `created_at`
- `metric_key`, `metric_value_bp`, `count_value`, `latency_ms`
- `evidence_ref` and `source_row_ref`

### 4.2 Modality Coverage
- STT: reject, timeout, low-confidence, disagreement, retry saturation.
- TTS: playback failure, policy block, stop ratio, completion ratio.
- NLP: clarify-loop rate, intent correction rate, ambiguity fallback rate.
- LLM: draft rejection rate, response correction rate, provider fallback rate.
- OCR: timeout rate, low-confidence rate, parse failure rate.
- Connectors: timeout/error rate, domain policy block rate, unavailable/provider-degraded rate.

### 4.3 Determinism Rules
- Windowing uses fixed buckets (`5m`, `1h`, `24h`) and UTC-aligned boundaries.
- Aggregation order is canonical (`tenant_id`, `modality`, `provider_id`, `domain_key`, `window_start`).
- All score inputs must be replayable from append-only rows.

## 5. Scorecards (Per Provider/Domain)

### 5.1 Scorecard Keys
- Primary key: `(tenant_id, modality, provider_id, domain_key, window_start, window_size)`
- Version key: `scorecard_version` increments on deterministic recompute drift.

### 5.2 Deterministic Metrics
- `success_rate_bp = ok_count * 10000 / total_count`
- `timeout_rate_bp = timeout_count * 10000 / total_count`
- `retry_exhaust_rate_bp = retry_exhaust_count * 10000 / total_count`
- `fallback_rate_bp = fallback_count * 10000 / total_count`
- `correction_rate_bp = correction_count * 10000 / total_count`
- `p95_latency_ms` and `p99_latency_ms` from deterministic percentile routine.
- `stability_score_bp` = weighted composite of above with fixed integer weights.

### 5.3 Threshold Packs
Each modality has deterministic threshold pack:
- `promote_min_sample_count`
- `assist_min_stability_bp`
- `lead_min_stability_bp`
- `demote_regression_bp`
- `hard_fail_timeout_bp`
- `hard_fail_error_bp`

Threshold packs are policy snapshots, never inferred at runtime.

## 6. Promotion Ladder

### 6.1 States
- `Shadow`: observe and report only.
- `Assist`: produce deterministic proposals, no direct mutation.
- `Lead`: may dispatch explicitly allowed LOW-risk policy commits through ACTIVE sims.

### 6.2 Promotion Rules
- Promote `Shadow -> Assist` when:
  - sample count minimum met,
  - stability over threshold for consecutive windows,
  - no hard-fail threshold breach.
- Promote `Assist -> Lead` when:
  - stricter threshold and longer stability horizon met,
  - rollback ref exists,
  - governance prerequisites satisfied.

### 6.3 Demotion and Rollback Rules
- Immediate demotion on hard-fail threshold breach.
- Deterministic rollback to previous policy snapshot on post-change regression.
- Out-of-order transitions fail closed.

## 7. AUTO_MICRO as Simulation Class (Not Engine)

### 7.1 Class Definition
Simulation class: `AUTO_MICRO`

Initial simulation IDs (design target):
- `AUTO_MICRO_REPORT_DRAFT`
- `AUTO_MICRO_POLICY_UPDATE_COMMIT`
- `AUTO_MICRO_POLICY_ROLLBACK_COMMIT`
- `AUTO_MICRO_PROMOTION_DECISION_COMMIT`

### 7.2 Autonomy Policy Snapshot (Required Fields)
- `snapshot_id`
- `tenant_id`
- `actor_user_id`
- `policy_version`
- `risk_tier`
- `allowed_action_kinds[]`
- `threshold_pack_ref`
- `frequency_cap_profile_ref`
- `rollback_plan_ref`
- `effective_at`
- `reason_code`

### 7.3 Frequency Caps
- Global cap: max AUTO_MICRO commits per tenant per 24h.
- Per-action cap: max commits per `(action_kind, provider_id/domain_key)` per rolling window.
- Cooldown cap: minimum interval between two commits for same target.
- Breach behavior: fail closed with explicit cap reason code.

### 7.4 Idempotency Recipes
- Report draft key: `(tenant_id, window_start, window_size, modality, provider_id, domain_key, scorecard_hash)`.
- Policy update key: `(tenant_id, action_kind, target_key, snapshot_id, scorecard_id, trigger_fingerprint)`.
- Promotion decision key: `(tenant_id, target_key, from_state, to_state, window_start, threshold_pack_ref)`.
- Rollback key: `(tenant_id, action_event_id, rollback_plan_ref, rollback_trigger_fingerprint)`.

### 7.5 Rollback References
Every update/promotion row must include:
- `rollback_ref` (pointer to exact previous active snapshot/value),
- `rollback_eligibility` boolean,
- `rollback_deadline` or `next_review_at`.

### 7.6 Proof Requirements
Required proof chain per committed action:
- `signal_set_ref`
- `scorecard_ref`
- `report_ref`
- one of:
  - `action_ref`
  - `proposal_ref`
  - `blocked_reason_ref`
- `access_decision_ref`
- `simulation_active_ref`
- `rollback_ref`

### 7.7 Access Action Registry (Fixed Strings)
All AUTO_MICRO runtime policy commits must use these exact access actions:
- `AUTO_MICRO_CONNECTOR_DISABLE_POLICY_UPDATE`
- `AUTO_MICRO_RETRY_TIMEOUT_POLICY_UPDATE`
- `AUTO_MICRO_POLICY_ROLLBACK`

## 8. “No Dead Reporting” Rule
Every report must terminate in one and only one actionable outcome:
- `ACTION_COMMIT_CANDIDATE` (LOW-risk and allowed), or
- `BUILDER_PROPOSAL_REQUIRED`, or
- `BLOCKED_WITH_REASON`.

Invalid states:
- Report generated without action/proposal/blocked reason.
- Threshold breach with no terminal reasoned outcome.

Enforcement:
- Report validation fails closed if terminal link missing.

## 9. Governance Boundaries
- AUTO_MICRO never bypasses governance, confirmation, or access boundaries.
- Runtime allow/deny uses per-user PH2 access instance only.
- Any non-LOW-risk change is builder-only activation path.
- LOW-risk actions are still simulation-gated, frequency-capped, idempotent, and rollback-linked.
- Policy snapshots are immutable after commit; new versions append-only.

Risk tiers:
- LOW: bounded temporary connector disable and bounded retry/timeout tuning within hard caps.
- MEDIUM/HIGH: any broader route behavior/policy impact, multi-provider global effects, or persistence-affecting behavior -> builder/governance flow only.

## 10. Two First Actions (Design Only)

### 10.1 Connector Temporary Disable Policy Commit
- Trigger signals:
  - sustained connector/provider failure or timeout rate above hard-fail threshold for N consecutive windows,
  - no conflicting safety/governance block,
  - minimum sample count satisfied.
- What it changes:
  - policy snapshot field for `(connector_or_provider, domain_key)`:
    - `enabled=false`
    - `disabled_until`
    - `disable_reason_code`
- Rollback behavior:
  - auto-expire at `disabled_until` with restore to previous snapshot, or
  - early rollback by `AUTO_MICRO_POLICY_ROLLBACK_COMMIT` on recovered scorecard.
- Default rollback timer:
  - `30 minutes` from commit when no earlier rollback trigger fires.
- Frequency caps:
  - max 1 disable per target per 15 minutes,
  - max 3 disables per target per 24h,
  - max 10 AUTO_MICRO LOW-risk actions per tenant per 24h.
- Required proof thresholds:
  - `sample_count >= promote_min_sample_count` for the target window,
  - `timeout_rate_bp >= hard_fail_timeout_bp` OR `error_rate_bp >= hard_fail_error_bp`,
  - threshold breach sustained for `N` consecutive windows,
  - no contradictory recovery signal in the same evaluation window.
- Required access action:
  - `AUTO_MICRO_CONNECTOR_DISABLE_POLICY_UPDATE`
- Required ACTIVE simulation IDs:
  - `AUTO_MICRO_POLICY_UPDATE_COMMIT`
  - `AUTO_MICRO_POLICY_ROLLBACK_COMMIT`

### 10.2 Retry/Timeout Budget Policy Commit
- Trigger signals:
  - timeout rate or retry exhaustion rate crossing threshold for N windows,
  - regression proof that current budget is suboptimal,
  - bounded impact domain.
- What it changes:
  - policy snapshot fields for `(modality, provider_id, domain_key)`:
    - `timeout_ms`
    - `retry_budget`
  - changes must remain inside hard global caps.
- Rollback behavior:
  - deterministic rollback to previous values on:
    - stability drop beyond demote threshold,
    - cap/safety violation,
    - no improvement after review window.
- Default rollback timer:
  - `60 minutes` from commit when improvement proof does not materialize.
- Frequency caps:
  - max 1 tuning commit per target per 30 minutes,
  - max 6 tuning commits per target per 24h.
- Required proof thresholds:
  - `sample_count >= promote_min_sample_count` for target `(modality, provider_id, domain_key)`,
  - `timeout_rate_bp >= hard_fail_timeout_bp` OR `retry_exhaust_rate_bp` over configured trigger threshold,
  - predicted post-change budget remains inside hard caps,
  - expected improvement delta is non-negative in deterministic score model.
- Required access action:
  - `AUTO_MICRO_RETRY_TIMEOUT_POLICY_UPDATE`
- Required ACTIVE simulation IDs:
  - `AUTO_MICRO_POLICY_UPDATE_COMMIT`
  - `AUTO_MICRO_POLICY_ROLLBACK_COMMIT`

## 11. Milestones (M0-M7)

| Milestone | Goal | Exact artifacts produced | Sims/policies affected | Acceptance tests required | Proof commands |
|---|---|---|---|---|---|
| M0 | Baseline audit and overlap map | `docs/35_AUTO_MICRO_BUILD_PLAN.md` overlap appendix, ownership map (`runtime vs builder`), gap list (`implemented / partial / missing`) | none | `AT-AUTO-MICRO-M0-01`, `AT-AUTO-MICRO-M0-02` | `bash scripts/check_auto_micro_overlap_map.sh` |
| M1 | Unified signal schema + writes (reporting only) | unified signal contract doc, storage write mapping spec, idempotency key spec, replay spec | policy snapshot schema draft only; no action sims | `AT-AUTO-MICRO-M1-01`, `AT-AUTO-MICRO-M1-02`, `AT-AUTO-MICRO-M1-03` | `cargo test -p selene_storage auto_micro_signal -- --nocapture` |
| M2 | Scorecards + thresholds + report generation | scorecard formula spec, threshold pack spec, report schema spec with terminal link field | `AUTO_MICRO_REPORT_DRAFT` (design target) | `AT-AUTO-MICRO-M2-01`, `AT-AUTO-MICRO-M2-02`, `AT-AUTO-MICRO-M2-03` | `cargo test -p selene_os auto_micro_scorecard -- --nocapture` |
| M3 | Builder proposal generation only | proposal payload spec, proposal ranking rules, blocked-reason schema | builder proposal path only, no direct commit action | `AT-AUTO-MICRO-M3-01`, `AT-AUTO-MICRO-M3-02`, `AT-AUTO-MICRO-M3-03` | `cargo test -p selene_os auto_micro_builder_proposal -- --nocapture` |
| M4 | Promotion ladder wiring | ladder transition spec, demotion/rollback triggers, transition proof schema | `AUTO_MICRO_PROMOTION_DECISION_COMMIT` (design target), policy snapshot versions | `AT-AUTO-MICRO-M4-01`, `AT-AUTO-MICRO-M4-02`, `AT-AUTO-MICRO-M4-03`, `AT-AUTO-MICRO-M4-04` | `cargo test -p selene_os auto_micro_ladder -- --nocapture` |
| M5 | Policy commits for first two actions | action payload specs, cap profiles, access action registry entries, commit/rollback recipe spec | `AUTO_MICRO_POLICY_UPDATE_COMMIT`, `AUTO_MICRO_POLICY_ROLLBACK_COMMIT`; access actions for connector disable + retry/timeout | `AT-AUTO-MICRO-M5-01`, `AT-AUTO-MICRO-M5-02`, `AT-AUTO-MICRO-M5-03`, `AT-AUTO-MICRO-M5-04` | `cargo test -p selene_os auto_micro_policy_commit -- --nocapture` |
| M6 | Rollback + cap enforcement + proof closure | rollback runbook, cap breach refusal matrix, end-to-end proof checklist | rollback/cap policies for AUTO_MICRO class | `AT-AUTO-MICRO-M6-01`, `AT-AUTO-MICRO-M6-02`, `AT-AUTO-MICRO-M6-03` | `cargo test -p selene_os auto_micro_rollback -- --nocapture` |
| M7 | CI guardrails | guardrail scripts + CI gate wiring for proofs/rollback/caps | CI policy gates only | `AT-AUTO-MICRO-M7-01`, `AT-AUTO-MICRO-M7-02`, `AT-AUTO-MICRO-M7-03` | `bash scripts/check_auto_micro_guardrails.sh` |

## 12. Acceptance Test Catalog
- `AT-AUTO-MICRO-M0-01-overlap-map-covers-pae-feedback-learn-builder-cost-cache-delivery-connector`
- `AT-AUTO-MICRO-M0-02-overlap-map-classifies-runtime-vs-builder-and-gating`
- `AT-AUTO-MICRO-M1-01-unified-signal-schema-validates-all-modalities`
- `AT-AUTO-MICRO-M1-02-signal-write-idempotent-no-duplicate-rows`
- `AT-AUTO-MICRO-M1-03-signal-replay-rebuild-equals-current-projection`
- `AT-AUTO-MICRO-M2-01-scorecard-deterministic-for-identical-input`
- `AT-AUTO-MICRO-M2-02-threshold-breach-generates-actionable-report`
- `AT-AUTO-MICRO-M2-03-no-dead-reporting-enforced`
- `AT-AUTO-MICRO-M3-01-proposal-only-no-direct-runtime-mutation`
- `AT-AUTO-MICRO-M3-02-proposal-includes-rollback-ref-and-idempotency`
- `AT-AUTO-MICRO-M3-03-blocked-reason-required-when-no-proposal`
- `AT-AUTO-MICRO-M4-01-shadow-to-assist-promotion-deterministic`
- `AT-AUTO-MICRO-M4-02-assist-to-lead-promotion-requires-thresholds-and-samples`
- `AT-AUTO-MICRO-M4-03-regression-demotes-and-links-rollback`
- `AT-AUTO-MICRO-M4-04-invalid-ladder-transition-fails-closed`
- `AT-AUTO-MICRO-M5-01-connector-disable-commit-requires-active-sim-and-access-allow`
- `AT-AUTO-MICRO-M5-02-retry-timeout-commit-requires-active-sim-and-access-allow`
- `AT-AUTO-MICRO-M5-03-policy-commit-enforces-frequency-caps`
- `AT-AUTO-MICRO-M5-04-policy-commit-idempotent`
- `AT-AUTO-MICRO-M6-01-rollback-restores-previous-policy-snapshot`
- `AT-AUTO-MICRO-M6-02-cap-violation-refuses-with-reason-code`
- `AT-AUTO-MICRO-M6-03-proof-chain-complete-signal-to-action-to-promotion`
- `AT-AUTO-MICRO-M7-01-ci-fails-on-missing-proof-links`
- `AT-AUTO-MICRO-M7-02-ci-fails-on-missing-rollback-links`
- `AT-AUTO-MICRO-M7-03-ci-fails-on-frequency-cap-bypass`

## 13. Implementation Guardrails for Future Packets
- Keep existing engine ownership: do not move PAE/FEEDBACK/LEARN/COST/CACHE logic into a new engine.
- Treat AUTO_MICRO as a policy/simulation orchestration class only.
- Add no activation path without ACTIVE simulation registration + per-user access ALLOW + proof chain.
- Keep all writes append-only + idempotent + replayable.

## 14. Data Model Appendix (Names Only)

### 14.1 Unified Signal Storage
- `auto_micro_signal_ledger`
- `auto_micro_signal_current`

### 14.2 Scorecard Storage
- `auto_micro_scorecard_ledger`
- `auto_micro_scorecard_current`

### 14.3 Policy Snapshot Storage
- `auto_micro_policy_snapshot_ledger`
- `auto_micro_policy_snapshot_current`

### 14.4 Promotion Decision Storage
- `auto_micro_promotion_decision_ledger`
- `auto_micro_promotion_decision_current`

### 14.5 Idempotency Indexes
- `auto_micro_signal_idempotency_index`
- `auto_micro_scorecard_idempotency_index`
- `auto_micro_policy_snapshot_idempotency_index`
- `auto_micro_promotion_decision_idempotency_index`
