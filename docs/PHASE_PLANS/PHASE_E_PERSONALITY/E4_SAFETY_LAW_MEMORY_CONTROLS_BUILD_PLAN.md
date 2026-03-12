PHASE E4 — SAFETY / LAW / MEMORY CONTROLS BUILD PLAN

A) REPO TRUTH CHECK
- `git status --short`: empty
- current branch: `main`
- HEAD commit at review start: `3801c7890abf1a376a3117da3bf4ee5d6ff28790`
- exact files reviewed:
  - `docs/CORE_ARCHITECTURE.md`
  - `docs/SELENE_BUILD_EXECUTION_ORDER.md`
  - `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`
  - `docs/PHASE_PLANS/PHASE_E_PERSONALITY/E1_PERSONALITY_ARCHITECTURE_REVIEW.md`
  - `docs/PHASE_PLANS/PHASE_E_PERSONALITY/E2_BOUNDED_ADAPTIVE_BEHAVIOR_MODEL_BUILD_PLAN.md`
  - `docs/PHASE_PLANS/PHASE_E_PERSONALITY/E3_TONE_VS_LONG_TERM_BEHAVIOR_SEPARATION_BUILD_PLAN.md`
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
- Freeze the canonical safety / law / memory control model that governs personality behavior, runtime tone, emotional modulation, bounded adaptation, and long-term behavior candidates.
- Preserve the frozen E1 architecture split, E2 adaptation bounds, and E3 tone-versus-behavior separation without reopening any authority or persistence rule.
- Define what is `ALLOWED`, `BOUNDED`, `DEGRADED`, `BLOCKED`, `FAIL_CLOSED`, `QUARANTINED`, or `SAFE_FAIL` for personality behavior and adaptive candidates.

C) DEPENDENCY RULE
- This phase consumes E1, E2, and E3 as frozen upstream law.
- This phase consumes Phase C authority / lifecycle / proof completion law and Phase D governance / runtime-law posture as reusable control patterns.
- E4 may define control classes, gating rules, and bounded gaps only. E4 may not redesign persona architecture, adaptation architecture, or persistence architecture.

D) ARCHITECTURAL POSITION
- Section 01 keeps deterministic execution and fail-closed requirements binding on all personality-affecting behavior that can influence execution posture.
- Section 04 establishes protected authority and permission boundaries when personality behavior intersects identity, sensitive content, or protected flows.
- Section 06 establishes memory suppression, retention, sensitivity, archive, and continuity constraints that bound personality influence from memory.
- Section 07 establishes identity and voice scope; personality behavior must remain identity-scoped and must not cross user or device identity boundaries.
- Section 08 establishes platform/runtime admissibility; device posture may narrow or block tone/adaptation behaviors.
- Section 09 establishes governance visibility and decision posture; governance does not become a persona writer.
- Section 10 provides deterministic scoring / threshold / decay mechanics where gating depends on computed confidence or severity.
- Section 11 establishes runtime-law response posture and fail-closed response classes; runtime law is the final posture layer, not a personality authoring layer.

E) E1 / E2 / E3 ASSUMPTIONS CONSUMED
- `PERSONA_CORE` remains authoritative and stable; tone, emotion, memory-fed personalization, and learning-fed adaptation may not rewrite it.
- Adaptive behavior remains bounded to frozen E2 allowed outputs and forbidden paths.
- Runtime tone remains transient by default; only bounded long-term behavior candidates may be nominated for later persistence consideration.
- Memory, feedback, and learning may nominate bounded candidates only; they may not become alternate personality authority.
- Governance, law, policy, identity, and platform signals remain control or decision layers only.

F) CURRENT CONTROL-RELEVANT SURFACES IN SCOPE
Current Repo Surface → E4 Control Scope Mapping

| repo surface | current role | authoritative / derived / evidence / decision | E4 relevance | notes / constraints |
| --- | --- | --- | --- | --- |
| `Ph1PersonaRepo` persisted profile snapshot and audit rows | persisted persona truth and audit lineage | authoritative | primary personality control anchor | may anchor allowed profile state; must not be rewritten by transient tone, emotion, feedback, or learning |
| PH1.PERSONA runtime outputs | runtime persona application surface | derived | subject to safety and law controls | must consume authoritative persona truth rather than rewrite it |
| PH1.EMO.CORE emotional signal bundle and guard flags | emotional-state signal generation | derived | bounded modulation input | may influence runtime tone only within control limits |
| PH1.EMO.GUIDE style modifiers and guide policies | tone and expression modifiers | derived | bounded style control input | may shape presentation but may not override persona core, policy, or law |
| PH1.CONTEXT bundles | context shaping and situational hints | derived | contextual gating input | bounded input only; may not override memory truth or policy/law posture |
| PH1.M continuity, preference, suppression, archive, retention surfaces | identity-scoped memory and privacy control substrate | authoritative in memory domain | memory influence and suppression gate | memory may nominate bounded behavior candidates; memory may not rewrite persona authority or identity truth |
| PH1.FEEDBACK signals | explicit user correction and feedback capture | evidence | bounded adaptation nomination input | feedback may nominate changes but may not directly rewrite persona core |
| PH1.LEARN artifacts | learned proposals and candidate artifacts | evidence / proposal | bounded long-term nomination input | requires bounded path; no direct promotion into authority |
| PH1.COMP deterministic scoring and threshold support | consensus / scoring / threshold computation | authoritative for deterministic computations | used where control class depends on scored signal | no free-form probabilistic control outcomes |
| PH1.POLICY decisions | policy control and permissions | decision | hard gate for allowed/bounded/blocked actions | policy is not a style engine |
| PH1.GOV decisions | governance visibility / decision posture | decision | protected-action and escalation gate | governance is not a persona writer |
| PH1.LAW posture | final runtime-law posture | decision | final block/degrade/quarantine/safe-fail posture | runtime law is not a style engine |
| identity / voice verification surfaces | identity, consent, speaker-scope, and trust gates | authoritative / decision | controls memory/personalization scope | no identity-mismatched personalization |
| platform/runtime posture surfaces | device/runtime trust and admissibility posture | decision / visibility | may degrade or block behavior | platform posture constrains behavior but does not become personality authority |

G) CANONICAL SAFETY / LAW / MEMORY CONTROL MODEL
- Personality-affecting actions are classified into control classes:
  - `ALLOWED`
  - `BOUNDED`
  - `DEGRADED`
  - `BLOCKED`
  - `FAIL_CLOSED`
  - `QUARANTINED`
  - `SAFE_FAIL`
- Control classes apply to:
  - runtime tone adjustments
  - emotional modulation adjustments
  - explanation-depth and acknowledgement changes
  - proactive suggestion intensity
  - memory-fed preference injection
  - feedback-fed nomination
  - learning-fed nomination
  - persisted behavior candidate promotion requests
- Control evaluation order is:
  - identity / platform admissibility
  - memory suppression / sensitivity / retention gate
  - policy gate
  - governance visibility / protected-action gate
  - runtime-law final posture
- If any required gate is missing, stale, or inconsistent, behavior must fail closed rather than continue optimistically.

H) PERSONALITY / ADAPTATION ACTION CONTROL CLASSES
Personality / Adaptation Action Control Matrix

| action or influence | control class | allowed / bounded / blocked | authoritative gate | visibility / decision surfaces | safe-fail rule | notes |
| --- | --- | --- | --- | --- | --- | --- |
| runtime tone adjustment | `BOUNDED` | allowed only within E2/E3 limits | persona core plus policy/law gate | PH1.POLICY, PH1.LAW | degrade or block if protected constraints are unresolved | must never change persona core |
| emotional modulation adjustment | `BOUNDED` | allowed only within runtime-tone limits | PH1.EMO.CORE plus policy/law gate | PH1.POLICY, PH1.LAW | reduce to neutral or bounded low-intensity mode on uncertainty | transient emotion may not promote itself to stable profile |
| explanation-depth adjustment | `BOUNDED` | allowed within deterministic bounds | E2 scoring threshold and policy gate | PH1.POLICY, PH1.LAW | clamp to neutral/default explanation depth if uncertain | must not suppress required compliance or safety content |
| proactive suggestion intensity adjustment | `BOUNDED` or `DEGRADED` | bounded by user preference, context, and policy | E2 scoring threshold and memory/policy gate | PH1.POLICY, PH1.GOV, PH1.LAW | degrade to low or none under uncertainty | cannot become workflow or authority selection |
| long-term behavior candidate nomination | `BOUNDED` | allowed as candidate only | persona authority plus bounded nomination path | PH1.FEEDBACK, PH1.LEARN, PH1.GOV | hold as candidate only; no direct promotion | persistence requires separate bounded path, never direct |
| memory-fed preference injection | `BOUNDED` / `BLOCKED` | bounded if memory scope is lawful; blocked if suppressed or out-of-scope | PH1.M identity/suppression/retention gate | PH1.M, PH1.POLICY, PH1.LAW | fail closed on missing identity scope or suppression ambiguity | may shape runtime tone but may not rewrite memory truth |
| feedback-fed behavior nomination | `BOUNDED` | allowed as nomination only | explicit feedback validity and policy gate | PH1.FEEDBACK, PH1.POLICY | no direct rewrite; unresolved cases remain candidate-only | user correction may bound future behavior but not rewrite persona core |
| learning-fed behavior nomination | `BOUNDED` / `QUARANTINED` | bounded candidate only; quarantined when provenance or scope is unclear | learn artifact trust and policy/law gate | PH1.LEARN, PH1.GOV, PH1.LAW | quarantine candidate on uncertain provenance | no direct promotion to stable behavior |
| persisted behavior promotion request | `BLOCKED` by default, `BOUNDED` only through explicit later path | blocked unless explicit bounded promotion path exists | persona authority plus policy/governance/law gate | PH1.POLICY, PH1.GOV, PH1.LAW | fail closed if bounded path is absent | current repo truth lacks a normalized persisted adaptation writer |
| identity-mismatched behavior personalization | `BLOCKED` | never allowed | identity/voice verification | identity surfaces, PH1.LAW | block and refuse personalization | no cross-user or mismatched-device persona adaptation |
| cross-user memory influence | `BLOCKED` | never allowed | identity scope and PH1.M privacy law | PH1.M, PH1.LAW | block and refuse | must never leak or transpose user memory |
| suppression-class memory influence | `BLOCKED` / `FAIL_CLOSED` | blocked when suppression exists; fail closed when suppression state is uncertain | PH1.M suppression rules | PH1.M, PH1.POLICY, PH1.LAW | refuse personalization using suppressed content | suppression dominates tone/adaptation convenience |

I) MEMORY INFLUENCE, SUPPRESSION, AND RETENTION CONTROLS
Memory Influence / Suppression Matrix

| memory concern | source surface | allowed personality effect | blocked effect | retention / sensitivity / suppression rule | notes |
| --- | --- | --- | --- | --- | --- |
| identity-scoped preference memory | PH1.M preference and continuity surfaces | bounded runtime personalization and candidate nomination | rewriting persona core or identity truth | preference use is identity-scoped and policy-gated | preference memory may shape tone only inside E2/E3 limits |
| suppression-class memory | PH1.M suppression rules | none beyond compliance-safe refusal handling | any personalization, tone shaping, or adaptation using suppressed content | `DO_NOT_MENTION`, `DO_NOT_REPEAT`, `DO_NOT_STORE` dominate | suppression is hard block, not advisory |
| sensitive retained memory | PH1.M sensitivity/retention posture | bounded recall only if lawful and identity-valid | broad style personalization or persistent behavior mutation | sensitivity and retention class constrain use, not just storage | sensitivity uncertainty must fail closed |
| archived or decayed memory | PH1.M archive/retention surfaces | at most bounded continuity if lawful and restored for use | implicit style carry-over from stale archives | archive/retention state must be explicit; no silent restore | stale or archived content may not silently shape behavior |
| cross-device memory continuity | PH1.M plus frozen Phase D session/identity law | identity-scoped continuity only | device-local mismatch or cross-user carry-over | cross-device carry-over must preserve identity scope and session law | memory continuity is bounded by D1-D5 session/identity rules |
| memory truth itself | PH1.M authoritative memory records | none; memory truth is control input only | any rewrite of memory truth by tone/adaptation/feedback/learn | no-memory-truth-rewrite is absolute | personality layers consume memory truth; they do not edit it |

J) IDENTITY / POLICY / GOVERNANCE / LAW CONTROL LAYERS
Identity / Policy / Governance / Law Constraint Matrix

| constraint source | affected layer or action | block / degrade / bounded modification / allow-with-warning | visibility / decision surface | notes |
| --- | --- | --- | --- | --- |
| identity / voice verification | memory-fed personalization, tone carry-over, long-term candidate nomination | `BLOCK` or `FAIL_CLOSED` on identity mismatch; bounded only on verified identity | identity/voice engine surfaces | no identity-mismatched personalization or candidate carry-over |
| platform/runtime posture | proactive intensity, emotional modulation, context-sensitive behavior | `DEGRADE`, `BLOCK`, or bounded modification depending runtime posture | platform/runtime posture surfaces | posture may narrow behavior even when persona inputs are otherwise lawful |
| PH1.POLICY | all adaptation and personality-affecting actions | `ALLOW`, `BOUNDED`, `BLOCK`, or `ALLOW_WITH_WARNING` | PH1.POLICY decision surfaces | policy is authoritative for permission class, not style authoring |
| PH1.GOV | protected or escalated behavior nominations and visibility-sensitive cases | bounded modification, `DEGRADE`, `BLOCK`, or `QUARANTINE` where governance visibility is required | PH1.GOV decision bundle / decision logs | governance is decision/visibility only, not a persona writer |
| PH1.LAW | final runtime posture across protected, uncertain, or safety-relevant cases | `ALLOW_WITH_WARNING`, `DEGRADE`, `BLOCK`, `QUARANTINE`, `SAFE_FAIL` | PH1.LAW posture surfaces | law is the final posture layer |
| PH1.M retention / suppression / sensitivity | memory-fed tone and candidate behavior | bounded modification or `BLOCK` | PH1.M authoritative memory surfaces | memory controls personality behavior but do not become persona authority |
| deterministic scoring layer | proactive intensity, explanation depth, carry-over thresholds | bounded modification or `DEGRADE` based on deterministic score | PH1.COMP / Section 10 computations | no free-form model discretion when scored gating is required |

K) FAILURE, REFUSAL, DEGRADATION, QUARANTINE, AND SAFE-FAIL MODEL
Failure / Refusal / Degradation Matrix

| case | preserved truth | runtime effect | governance / law effect | safe-fail result | notes |
| --- | --- | --- | --- | --- | --- |
| missing identity scope for personalization | persona core and memory truth stay unchanged | personalization refused | law may block or fail closed | neutral/non-personalized behavior only | no guesswork on user scope |
| suppression state missing or conflicting | memory truth and suppression intent stay authoritative | memory-fed influence refused | policy/law visibility may be required | fail closed on memory use | suppression ambiguity is not recoverable by tone heuristics |
| policy denies adaptive output | persona and runtime state remain unchanged except refusal posture | requested output blocked or degraded | policy decision visible; law may reinforce | bounded neutral response only | policy cannot be bypassed by tone/adaptation |
| governance visibility required but unavailable | authoritative persona/memory truth preserved | behavior degraded or blocked | governance effect unresolved; law may quarantine | bounded safe response only | no protected completion without required visibility |
| runtime-law posture unresolved or blocking | authoritative truth preserved | action blocked, degraded, quarantined, or safe-failed | PH1.LAW final response class | safe-fail response only | law is final posture, not advisory |
| learning artifact provenance unclear | persona core and candidate state preserved | no promotion; candidate quarantined | governance/law may mark uncertain | quarantine nomination | uncertain learning never promotes itself |
| feedback conflicts with persona or policy | persona core preserved; feedback stays evidence | runtime may acknowledge but not apply direct rewrite | policy/governance/law may require bounded handling | candidate-only, no direct rewrite | explicit user correction is evidence, not authority |
| deterministic score below threshold | no behavior drift | runtime falls back to neutral or bounded default | none or bounded visibility only | degrade to default behavior | low-confidence adaptation never escalates privilege |

L) NO-AUTHORITY-BLEED / NO-MEMORY-TRUTH-REWRITE RULES
No-Authority-Bleed / No-Memory-Rewrite Matrix

| forbidden mutation | why forbidden | governing source | must never be treated as | notes |
| --- | --- | --- | --- | --- |
| tone overriding persona core | transient presentation may not rewrite stable persona authority | E1 persona authority law | persona authoring | tone is runtime-only presentation |
| emotion rewriting stable profile | transient emotional state is not long-term identity | E1/E3 layer separation | stable trait promotion | emotion can modulate runtime expression only |
| memory rewriting memory truth or identity truth | memory is authoritative in its own domain and identity is separately authoritative | PH1.M plus identity law | self-editing memory authority | memory influence may shape behavior; it may not mutate truth |
| feedback directly rewriting persona core | feedback is evidence and nomination, not persona authority | E2 bounded adaptation law | direct persona write | feedback must pass bounded candidate path |
| learning directly promoting long-term behavior without bounded path | learning artifacts are proposals, not stable authority | E2 bounded adaptation law | direct persisted behavior writer | requires bounded promotion path, currently absent |
| policy / governance / law being treated as style engines | control layers decide permission/posture, not presentation authorship | PH1.POLICY / PH1.GOV / PH1.LAW | style-generation authority | they constrain or block; they do not style-author |
| adaptation changing workflow / authority / simulation selection | personality system cannot alter deterministic execution authority | Core architecture / Phase C/D authority law | execution dispatcher | adaptive behavior must never bypass deterministic control layers |

M) E4 → E5 FREEZE BOUNDARY
E4 → E5 Boundary Matrix

| concern | frozen in E4 | deferred to E5 | rationale |
| --- | --- | --- | --- |
| action control classes | yes | verification only | E5 may test that `ALLOWED`, `BOUNDED`, `DEGRADED`, `BLOCKED`, `FAIL_CLOSED`, `QUARANTINED`, and `SAFE_FAIL` are enforced |
| memory suppression / retention / sensitivity control law | yes | verification only | E5 may verify suppression, retention, and identity-scope outcomes |
| identity / policy / governance / law control layering | yes | verification only | E5 may verify routing and evidence, not redesign control order |
| no-authority-bleed / no-memory-truth-rewrite rules | yes | verification only | E5 may prove mutation is prevented, not reinterpret the rule |
| deterministic scored gating | yes | threshold verification only | E5 may validate threshold behavior and reset/degrade outcomes |
| bounded repo-truth gaps | yes | document and verify gap handling only | E5 may confirm fail-closed handling for still-missing materialization paths |

N) COMPLETION CRITERIA
- The control classes for all personality and adaptation actions are frozen with no ambiguity about which actions are `ALLOWED`, `BOUNDED`, `DEGRADED`, `BLOCKED`, `FAIL_CLOSED`, `QUARANTINED`, or `SAFE_FAIL`.
- Memory suppression, retention, sensitivity, identity-scope, and no-memory-truth-rewrite law are frozen with no ambiguity.
- Policy, governance, and runtime-law control roles are frozen as control/decision layers only, with no authority bleed into persona authoring.
- Runtime-only versus persisted-candidate guarding is frozen without reopening E2 or E3.
- The repo-truth gaps are explicitly listed as bounded gaps rather than silently interpreted as existing control writers.
- E5 is limited to verification, evidence, docs, and closure work; it may not reinterpret E4 control law.
