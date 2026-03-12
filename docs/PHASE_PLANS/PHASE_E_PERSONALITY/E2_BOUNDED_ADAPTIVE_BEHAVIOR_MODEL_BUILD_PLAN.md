PHASE E2 — BOUNDED ADAPTIVE BEHAVIOR MODEL BUILD PLAN

A) REPO TRUTH CHECK
- `git status --short`: empty
- current branch: `main`
- HEAD commit at start: `e3063c72b28a2818a866ede6fd9fd7c1c3df9907`
- target file: `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/PHASE_PLANS/PHASE_E_PERSONALITY/E2_BOUNDED_ADAPTIVE_BEHAVIOR_MODEL_BUILD_PLAN.md`
- exact files reviewed:
  - `docs/CORE_ARCHITECTURE.md`
  - `docs/SELENE_BUILD_EXECUTION_ORDER.md`
  - `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`
  - `docs/PHASE_PLANS/PHASE_E_PERSONALITY/E1_PERSONALITY_ARCHITECTURE_REVIEW.md`
  - `docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C1_LIFECYCLE_MODEL_REVIEW.md`
  - `docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C3_MEMORY_RETENTION_PURGE_DELETE_ENFORCEMENT_BUILD_PLAN.md`
  - `docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C4_STORAGE_PROOF_LAW_INTEGRATION_BUILD_PLAN.md`
  - `docs/PHASE_PLANS/PHASE_C_LIFECYCLE/C5_TESTS_DOCS_VERIFICATION_BUILD_PLAN.md`
  - `docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D1_CROSS_DEVICE_SESSION_CONSISTENCY_REVIEW.md`
  - `docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D4_LAW_GOVERNANCE_ALIGNMENT_BUILD_PLAN.md`
  - `docs/PHASE_PLANS/PHASE_D_CROSS_DEVICE_SESSION/D5_TESTS_DOCS_VERIFICATION_BUILD_PLAN.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_01.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_04.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_06.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_07.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_08.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_09.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_10.md`
  - `docs/BUILD_SECTIONS/SELENE_BUILD_SECTION_11.md`
  - `docs/DB_WIRING/PH1_PERSONA.md`
  - `docs/DB_WIRING/PH1_EMO.md`
  - `docs/DB_WIRING/PH1_EMO_CORE.md`
  - `docs/DB_WIRING/PH1_EMO_GUIDE.md`
  - `docs/DB_WIRING/PH1_CONTEXT.md`
  - `docs/DB_WIRING/PH1_M.md`
  - `docs/DB_WIRING/PH1_LEARN.md`
  - `docs/DB_WIRING/PH1_POLICY.md`
  - `docs/DB_WIRING/PH1_GOV.md`
  - `docs/DB_WIRING/PH1_LAW.md`
  - `docs/DB_WIRING/PH1_COMP.md`
  - `docs/DB_WIRING/PH1_FEEDBACK.md`
  - `crates/selene_kernel_contracts/src/ph1persona.rs`
  - `crates/selene_kernel_contracts/src/ph1e.rs`
  - `crates/selene_kernel_contracts/src/ph1emocore.rs`
  - `crates/selene_kernel_contracts/src/ph1emoguide.rs`
  - `crates/selene_kernel_contracts/src/ph1context.rs`
  - `crates/selene_kernel_contracts/src/ph1m.rs`
  - `crates/selene_kernel_contracts/src/ph1learn.rs`
  - `crates/selene_kernel_contracts/src/ph1policy.rs`
  - `crates/selene_kernel_contracts/src/ph1comp.rs`
  - `crates/selene_kernel_contracts/src/ph1feedback.rs`
  - `crates/selene_kernel_contracts/src/runtime_governance.rs`
  - `crates/selene_kernel_contracts/src/runtime_law.rs`
  - `crates/selene_storage/src/repo.rs`
  - `crates/selene_storage/src/lib.rs`

B) PURPOSE
- Freeze the bounded adaptive behavior model that downstream Phase E work must consume.
- Separate lawful adaptive inputs and outputs from forbidden adaptation paths.
- Preserve E1 personality architecture, Phase C authority law, and Phase D session/governance/law boundaries.

C) DEPENDENCY RULE
- E2 depends on frozen E1 layer boundaries and may not reinterpret them.
- E2 depends on Phase C authority, lifecycle, and completion law when adaptation proposals touch persisted state, evidence, or protected completion.
- E2 depends on Phase D authority, consistency, governance, and law posture so adaptive behavior never reopens session authority or conflict law.
- If an adaptive path would change persona authority, identity truth, memory truth, governance posture, or runtime-law posture, it is out of scope for E2 and must fail closed.

D) ARCHITECTURAL POSITION
- E2 defines bounded adaptive behavior only.
- E2 does not redesign personality architecture; E1 already froze that model.
- E2 does not materialize tone-vs-long-term persistence splits; that is deferred to E3.
- E2 does not design safety, law, memory, or control enforcement logic; that is deferred to E4.
- E2 does not define tests or closure execution; that is deferred to E5.

E) E1 ASSUMPTIONS CONSUMED
- `PERSONA_CORE` stays stable and is not rewritten by adaptive logic.
- `TONE_PRESENTATION_LAYER` is the main lawful adaptive surface.
- `EMOTIONAL_MODULATION_LAYER` may modulate delivery but may not change meaning or authority.
- `MEMORY_FED_PERSONALIZATION_LAYER` is advisory and identity-scoped.
- `LEARNING_FED_ADAPTATION_LAYER` is proposal-only and non-authoritative.
- `CONSTRAINT_LAYER` from policy, governance, law, identity, and platform posture always outranks adaptive preference.

F) CURRENT ADAPTATION-RELEVANT SURFACES IN SCOPE
Current Repo Surface → E2 Adaptive Scope Mapping

| repo surface | current role | authoritative / derived / evidence / decision | adaptive relevance | notes / constraints |
| --- | --- | --- | --- | --- |
| `Ph1PersonaRepo` profile snapshot rows plus `style_profile_ref` / `delivery_policy_ref` / `preferences_snapshot_ref` | persisted persona profile baseline | authoritative within persona profile domain | provides stable persona and preference baseline | must not be rewritten directly by runtime adaptation |
| `PH1.PERSONA` runtime personalization outputs | personality presentation assembly | derived / advisory | lawful adaptive output carrier | guarded by `tone_only`, `no_meaning_drift`, `no_execution_authority` |
| `PH1.EMO.CORE` emotional signal bundle | bounded emotional state computation | derived / advisory | lawful adaptive input | requires identity and consent; non-authoritative |
| `PH1.EMO.GUIDE` style profile and modifiers | emotional tone guide | derived / advisory | lawful adaptive input and style cap | may influence delivery only |
| `PH1.CONTEXT` bundles | bounded context assembly | derived / advisory | lawful adaptive input | no direct authority; validation must pass |
| `PH1.M` memory preference / continuity surfaces | identity-scoped memory truth and preference continuity | authoritative in memory domain | lawful adaptive input with strict scope limits | may not rewrite personality authority |
| `PH1.FEEDBACK` signal bundles | structured correction / confidence / preference feedback | evidence / advisory | lawful adaptive input candidate | may not directly mutate runtime authority |
| `PH1.LEARN` artifacts | learning package proposals | derived / proposal | lawful adaptive input candidate only if consent-safe and derived-only | no direct runtime activation path |
| `PH1.COMP` deterministic scoring outputs | deterministic quantitative computation | derived compute artifact | provides scoring / threshold / rank support | computes only; never authorizes |
| `PH1.POLICY` decisions | policy constraint layer | decision | adaptive hard-boundary constraint | outputs decisions only |
| `PH1.GOV` decisions | governance visibility / decision layer | decision | adaptive escalation / visibility constraint | never personality writer |
| `PH1.LAW` final runtime posture | final law posture | decision | adaptive hard-stop / degrade / block constraint | identical inputs must yield identical outputs |
| identity and voice verification surfaces | user identity and voice truth | authoritative in identity domain | adaptive gating input | unknown or unverified identity fails closed |
| platform posture surfaces | runtime environment admissibility / posture | decision / visibility | adaptive gating input | low-trust platform posture may only restrict, never broaden adaptation |

G) CANONICAL BOUNDED ADAPTIVE BEHAVIOR MODEL
- E2 defines bounded adaptation as deterministic, constrained modulation of delivery behavior using lawful, evidence-backed, identity-scoped, and policy-compliant signals.
- Canonical adaptive decision order is:
  1. verify identity / consent / policy / law / governance posture prerequisites
  2. collect lawful advisory inputs
  3. apply deterministic scoring / threshold / decay rules where quantitative gating is required
  4. produce bounded runtime adaptation outputs only within E1 layer limits
  5. refuse, degrade, or reset adaptation when any governing constraint fails
- Adaptive behavior must never:
  - rewrite `PERSONA_CORE`
  - rewrite memory truth
  - rewrite identity truth
  - alter workflow / authority / simulation selection
  - override policy, governance, or law posture
  - introduce unbounded self-modification

H) ADAPTIVE SIGNAL INPUTS AND ELIGIBILITY
Adaptive Signal Eligibility Matrix

| signal | source surface | allowed or forbidden | confidence / threshold basis if any | notes |
| --- | --- | --- | --- | --- |
| persona profile input | `Ph1PersonaRepo` persisted profile snapshot and PH1.PERSONA profile refs | allowed | none by default; stable baseline input | identity-verified baseline only |
| emotional-state input | `PH1.EMO.CORE` signal bundle (`assertive_score`, `distress_score`, `anger_score`, `warmth_signal`) | allowed | bounded by existing validation caps and consent / identity requirements | no authority effect |
| emotional-guide input | `PH1.EMO.GUIDE` style profile and modifiers | allowed | no numeric threshold unless PH1.COMP gates a downstream choice | style-only input |
| memory-fed preference input | PH1.M preference / continuity surfaces | allowed | must be identity-scoped and suppression-compliant | no cross-user memory reads |
| context-bundle input | `PH1.CONTEXT` bundle | allowed | only after bundle validation succeeds | advisory only |
| explicit feedback input | `feedback_signal_bundle` from PH1.FEEDBACK | allowed | use deterministic confidence bucket / threshold where adaptation depends on confidence | feedback influences candidates only |
| learning artifact input | PH1.LEARN persona-delta / package artifact | allowed only as proposal input | must satisfy `consent_safe`, `derived_only_global`, and threshold / confidence checks | never direct activation |
| policy / law / governance signals | PH1.POLICY / PH1.GOV / PH1.LAW decisions | allowed as constraints only | binary or posture-gated, not preference-scored | may reduce or block adaptation only |
| identity signal | identity / voice verification surfaces | allowed as gating prerequisite only | verification success required | missing verification blocks adaptive use |
| platform posture signal | platform runtime posture | allowed as gating prerequisite only | deterministic posture class threshold if present | low-trust posture degrades or blocks |
| user correction or suppression-class signal | PH1.FEEDBACK plus PH1.M suppression / correction surfaces | allowed | evidence-backed and deterministic confidence threshold required | may suppress or reset adaptation |
| raw cross-user memory | any non-identity-scoped PH1.M or external memory read | forbidden | n/a | violates identity and privacy law |
| unsourced inferred preference | runtime guess without evidence-backed surface | forbidden | n/a | guessing is forbidden |

I) ALLOWED ADAPTIVE OUTPUTS AND HARD LIMITS
Allowed Adaptive Output Matrix

| adaptive output | source layer affected | allowed range | runtime-only or persistable candidate | hard limit | notes |
| --- | --- | --- | --- | --- | --- |
| phrasing / tone presentation | `TONE_PRESENTATION_LAYER` | style selection, modifier weighting, delivery tone only | runtime-only by default; persisted candidate only via later governed profile path | may not change factual meaning or intent | must preserve `tone_only` |
| pacing / verbosity | `TONE_PRESENTATION_LAYER` | bounded shorter / longer response presentation | runtime-only by default; persisted candidate only as profile preference later | may not suppress required warnings, confirmations, or disclosures | no authority drift |
| emotional modulation strength | `EMOTIONAL_MODULATION_LAYER` | bounded intensity scaling within validated emo-core ranges | runtime-only; persisted form forbidden without later governed design | may not exceed validated score caps or override law/policy posture | consent and identity required |
| explanation depth | presentation and context use only | bounded elaboration / summarization | runtime-only by default; persisted candidate only as stable preference later | may not omit required safety, policy, or execution details | no semantic drift |
| acknowledgement style | presentation and social framing | bounded acknowledgment wording | runtime-only by default | may not fabricate memory, confidence, or authority | no false certainty |
| proactive suggestion intensity | presentation and recommendation framing | bounded prompt / follow-up suggestion strength | runtime-only by default; persisted candidate only via later profile preference path | may not change workflow authority or trigger hidden actions | suggestion only |

J) DISALLOWED ADAPTATION PATHS
Disallowed Adaptation Path Matrix

| forbidden adaptation | why forbidden | governing source | must never be treated as | notes |
| --- | --- | --- | --- | --- |
| persona core rewrite | E1 froze `PERSONA_CORE` as stable | E1, PH1.PERSONA authority split | normal adaptive tuning | requires explicit later governed profile change path |
| memory truth rewrite | memory is authoritative in PH1.M | Section 06, PH1.M, Phase C authority law | adaptive preference update | adaptation may consume memory hints only |
| identity truth rewrite | identity is authoritative | Section 07 identity law | personalization update | unknown identity fails closed |
| policy override | PH1.POLICY decisions outrank adaptation | PH1.POLICY, governance/law constraints | strong preference satisfaction | decisions only, no bypass |
| law posture override | PH1.LAW is final runtime posture | Section 11, PH1.LAW | tone preference | law may degrade or block adaptation |
| authority / workflow / simulation selection drift | adaptation must never affect deterministic execution authority | repository architecture law, Section 01, Phase C / D baselines | proactive helpfulness | execution remains non-probabilistic |
| unbounded long-term self-modification | long-term adaptive balancing remains unimplemented and bounded | CORE_ARCHITECTURE, E1 | learning success | only bounded governed candidate persistence later |

K) ADAPTATION SCORING, THRESHOLDS, AND DECAY MODEL
- Deterministic quantitative gating is owned by PH1.COMP and Section 10 computation law whenever adaptation depends on ranking, thresholding, confidence, or decay.
- E2 requires scoring and thresholds to be deterministic, auditable, replay-safe, and cloud-reproducible.
- If a lawful adaptive decision needs quantitative gating, the minimum rule is:
  - compute score from evidence-backed inputs only
  - compare `score_bp` to `threshold_bp`
  - require explicit minimum confidence bucket or equivalent deterministic confidence gate
  - refuse or degrade when confidence is below threshold
  - reset or decay when supporting evidence weakens, consent is revoked, identity changes, or suppression-class feedback arrives

Adaptation Scoring / Threshold Matrix

| adaptive decision | input set | deterministic scoring or threshold rule | minimum confidence required | decay or reset rule | notes |
| --- | --- | --- | --- | --- | --- |
| tone modifier selection | persona profile + emo-core + emo-guide | deterministic rank ordering of style modifiers; if quantitative tie-break needed, PH1.COMP computes stable score | medium-or-higher confidence when feedback-driven | reset on identity change, consent loss, or suppression-class signal | style-only effect |
| verbosity adaptation | persona preference + explicit feedback + context bundle | `score_bp >= threshold_bp` using explicit feedback and prior preference evidence only | explicit confidence bucket above low | decay toward stable profile when fresh support absent | must not remove required content |
| explanation-depth adaptation | persona preference + explicit correction + context bundle | deterministic threshold gate; if confidence low, keep baseline explanation depth | medium confidence | reset on contradictory feedback or session reset | no meaning drift |
| emotional modulation strength | emo-core signals + consent + identity verification | bounded scalar selection within validated caps; if computed, PH1.COMP supplies rank/threshold | identity verified and consent asserted | reset immediately when consent or identity invalid | runtime-only |
| proactive suggestion intensity | persona preference + explicit feedback + policy/law posture | deterministic threshold plus hard policy/law cap | medium confidence and no law/policy block | decay to conservative baseline on stale support | never changes workflow authority |
| learn-package candidate persistence readiness | explicit feedback + learn artifact metadata | `score_bp` / `threshold_bp` from PH1.LEARN plus `consent_safe` and `derived_only_global` checks | threshold must be met exactly | reset candidate when no-runtime-drift or consent-safe check fails | still non-authoritative proposal only |

L) GOVERNANCE / LAW / MEMORY / IDENTITY / POLICY CONSTRAINTS
Governance / Law / Memory / Identity / Policy Constraint Matrix

| constraint source | adaptive scope affected | hard block or bounded modification | visibility / decision surface | notes |
| --- | --- | --- | --- | --- |
| identity verification | all adaptive inputs and outputs | hard block when identity missing or mismatched | identity / voice verification surfaces | no adaptive use without identity truth |
| PH1.M memory suppression / privacy / retention | memory-fed personalization and persisted-candidate adaptation | hard block on suppressed or out-of-scope memory; bounded modification otherwise | PH1.M memory surfaces | memory may constrain but not author personality authority |
| PH1.POLICY | all adaptive outputs | hard block or bounded modification | policy decisions | policy decisions outrank preference |
| PH1.GOV | escalation-sensitive or governed adaptation classes | bounded modification or block; may require visibility before persistable-candidate promotion | governance decision surfaces | governance never becomes persona writer |
| PH1.LAW | any adaptive path affecting protected runtime posture | hard block, degrade, or safe-mode posture | law posture surfaces | final runtime posture only |
| platform posture | emotional modulation and suggestion intensity | bounded modification or block | platform runtime posture | low-trust device may reduce adaptation |

M) RUNTIME VS PERSISTED ADAPTATION BOUNDARY
- Runtime adaptation is the default E2 mode.
- Persisted adaptation is not directly authored by E2; E2 only defines what may later become a governed persisted candidate.
- Any candidate-for-persistence must remain advisory until a later explicit profile-governed path exists.
- Raw emotional volatility, raw feedback events, raw memory truth, law posture, governance posture, and identity truth must never be persisted as adaptive personality state.

Runtime vs Persisted Adaptation Boundary Matrix

| adaptive aspect | runtime-only | candidate for persisted adaptation | forbidden persisted form | downstream phase most affected | notes |
| --- | --- | --- | --- | --- | --- |
| phrasing / tone presentation | yes by default | yes, only as later persona profile preference candidate | direct mutation of persona core or meaning-bearing behavior | E3 | presentation only |
| pacing / verbosity | yes by default | yes, only as later stable preference candidate | raw per-turn volatility history as authority | E3 | bounded preference only |
| emotional modulation strength | yes | no by default | raw emotional state as long-term persona truth | E4 | ephemeral and consent-gated |
| explanation depth | yes by default | yes, only as later governed profile preference candidate | direct workflow / authority change | E3 | must preserve required disclosures |
| acknowledgement style | yes | limited later profile preference candidate only | memory truth rewrite or fabricated familiarity | E3 | social framing only |
| proactive suggestion intensity | yes | limited later profile preference candidate only | autonomous workflow selection drift | E3 / E4 | suggestion-only effect |
| learn-package adaptation candidate | no direct runtime activation | yes, proposal-only candidate | active personality mutation without governed path | E3 / E4 | remains non-authoritative |

N) E2 → E3 / E4 / E5 FREEZE BOUNDARY
E2 → E3 / E4 / E5 Boundary Matrix

| concern | frozen in E2 | deferred to E3 | deferred to E4 | deferred to E5 | rationale |
| --- | --- | --- | --- | --- | --- |
| lawful adaptive inputs | exact eligibility and forbidden-input rules | no reinterpretation | no reinterpretation | verify only | prevents hidden input expansion |
| allowed adaptive outputs | exact bounded output list and hard limits | materialize tone-vs-long-term separation only | enforce safety controls only | verify only | keeps downstream from widening output scope |
| disallowed adaptation paths | exact forbidden set | no reinterpretation | may enforce stronger controls, not weaken | verify only | protects persona core and authority boundaries |
| deterministic scoring / thresholds / decay | exact deterministic gating requirement | may materialize persistence path inputs | may add control gating where law/policy require | verify only | prevents probabilistic adaptation drift |
| runtime vs persisted boundary | runtime-only default and candidate-for-persisted rules | may design persistence materialization only | may constrain persistability further | verify only | stops direct persistence drift |
| governance / law / memory / identity / policy constraints | exact constraint precedence | no reinterpretation | may implement and harden controls only | verify only | preserves authority and safety ordering |

O) COMPLETION CRITERIA
- E2 is complete when all lawful adaptive inputs, forbidden inputs, allowed outputs, forbidden outputs, deterministic gating rules, hard constraints, and runtime-vs-persisted boundaries are frozen in one approval-grade plan.
- E2 is not complete if downstream phases would still need to guess:
  - whether an input is lawful
  - whether an output may adapt
  - what hard limit applies
  - whether a quantitative threshold is required
  - whether a signal is runtime-only or candidate-for-persisted adaptation
  - whether governance, law, memory, identity, or policy may block the path
- E3 may only materialize separation and persistence mechanics inside the E2 bounds.
- E4 may only add control and enforcement logic inside the E2 bounds.
- E5 may only verify, document, and close the E2 commitments.
