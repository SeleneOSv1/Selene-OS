# PH1_LEARN DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.LEARN
- layer: Learning Assist
- authority: Non-Authoritative
- role: Learning signal aggregation and adaptation artifact package building
- placement: TURN_OPTIONAL (post-turn async)

## B) Ownership
- Tables owned: NONE (vNext runtime design)
- Reads:
  - Structured, OS-supplied feedback/listen signals and bounded evidence references.
  - Optional offline pattern/RLL proposal hints after governance activation.
- Writes: NONE (no direct persistence in vNext runtime wiring)
- Persistence boundary:
  - Append-only artifact/audit storage semantics remain canonical in `docs/DB_WIRING/PH1_LEARN_FEEDBACK_KNOW.md`.

## C) Hard Boundaries
- Non-authoritative and non-executing; outputs are advisory package candidates only.
- Must never grant authority, alter permissions, or bypass Access + Simulation ordering.
- Must never perform side effects, tool execution, or engine-to-engine direct calls.
- Hard rule: consent-required signals without asserted consent fail closed.
- Hard rule: global-derived signals must remain derived-only and consent-safe.
- Hard rule: LEARN artifacts are package proposals only; no runtime activation path exists inside PH1.LEARN.

## D) Wiring
- Invoked_by: Selene OS in post-turn learning/adaptation window.
- Inputs_from:
  - PH1.FEEDBACK validated signal output (`FEEDBACK_SIGNAL_EMIT=OK`).
  - PH1.LISTEN validated adaptation signals (`LISTEN_SIGNAL_FILTER=OK`) when available.
  - Governed artifacts from offline pipeline (`PH1.PATTERN`/`PH1.RLL`) only after activation.
- Outputs_to:
  - Ordered LEARN artifact candidates and selected package id for Selene OS.
  - Validated target-engine routing hints for PH1.PERSONA/PH1.KNOW/PH1.PAE/PH1.CACHE/PH1.PRUNE/PH1.SEARCH/PH1.LISTEN.
- Invocation_condition: OPTIONAL (learning policy enabled)
- Deterministic sequence:
  - `LEARN_SIGNAL_AGGREGATE`
    - validates consent + scope boundaries.
    - derives deterministic ordered artifact candidates + selected artifact id.
  - `LEARN_ARTIFACT_PACKAGE_BUILD`
    - validates canonical ordering, rollback/versioning discipline, and target-engine coverage.
    - returns `validation_status (OK|FAIL)` with bounded diagnostics.
  - If `validation_status != OK`, Selene OS fails closed and does not forward LEARN artifacts.
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Related Engine Boundaries
- PH1.FEEDBACK outputs are LEARN inputs only after FEEDBACK validation succeeds.
- PH1.PAE/PH1.KNOW/PH1.CACHE may consume LEARN output only as advisory governed artifacts; no direct execution path.
- PH1.PERSONA may consume LEARN persona-delta artifacts only as advisory profile input; no direct execution path.
- PH1.PAE consumption path is lock-ordered: `PAE_POLICY_SCORE_BUILD` may read LEARN package signals, then `PAE_ADAPTATION_HINT_EMIT` may emit downstream hints only after score-order validation.
- PH1.PATTERN and PH1.RLL remain offline-only proposal/ranking engines; LEARN runtime must not invoke them directly.
- PH1.LEARN runtime contracts are separate from combined storage contract (`PH1_LEARN_FEEDBACK_KNOW`) and do not replace append-only persistence rules.

## F) Acceptance Tests
- AT-LEARN-01: Selene OS can invoke `LEARN_SIGNAL_AGGREGATE` and output is schema-valid.
- AT-LEARN-02: Consent-required signals without asserted consent fail closed deterministically.
- AT-LEARN-03: `LEARN_ARTIFACT_PACKAGE_BUILD` fails closed on ordering/target drift.
- AT-LEARN-04: Valid package build returns `validation_status=OK` with deterministic target-engine coverage.

## G) FDX Wiring Lock (Section 5F)
- PH1.LEARN wiring must aggregate FDX signals into versioned, rollbackable artifact packages.
- FDX artifact targets may include PH1.K/PH1.C/PH1.NLP/PH1.X/PH1.TTS adaptation surfaces only as advisory packages.
- PH1.LEARN must fail closed on invalid ordering, invalid rollback pointers, or invalid target-engine coverage.

## H) Round-2 Step 9 Lock (Feedback-Driven Learning Package Flow)
- PH1.LEARN input route from PH1.FEEDBACK is now explicitly locked in Selene OS runtime:
  - canonical FEEDBACK candidate ordering before LearnSignal creation
  - stable deterministic `signal_id` projection for replay-safe idempotent processing
  - strict fail-closed correlation/turn/tenant integrity checks
- PH1.LEARN package output authority remains unchanged:
  - advisory-only
  - no execution authority
  - any runtime activation remains governed by PH1.PAE promotion path

## I) Round-2 Step 10 Lock (Gold-Loop Verified Packaging)
- Improvement-path FEEDBACK signals from PH1.C miss/correction gold-cases are accepted into LEARN routing with deterministic signal mapping.
- PH1.LEARN package-build remains fail-closed unless improvement artifacts are gold-verified:
  - `gold_status=VERIFIED`
  - provenance method present
  - gold-case id present
- Verified gold-loop outputs must preserve deterministic replay identity and target PAE when FEEDBACK target includes `PaeScorecard`.
