PHASE E3 — TONE VS LONG-TERM BEHAVIOR SEPARATION BUILD PLAN

A) REPO TRUTH CHECK
- `git status --short`: empty
- current branch: `main`
- repo-truth snapshot HEAD at review start: `36b10235788dd4594108f4f42a311dc3ca820b9c`
- exact files reviewed:
  - `docs/CORE_ARCHITECTURE.md`
  - `docs/SELENE_BUILD_EXECUTION_ORDER.md`
  - `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`
  - `docs/PHASE_PLANS/PHASE_E_PERSONALITY/E1_PERSONALITY_ARCHITECTURE_REVIEW.md`
  - `docs/PHASE_PLANS/PHASE_E_PERSONALITY/E2_BOUNDED_ADAPTIVE_BEHAVIOR_MODEL_BUILD_PLAN.md`
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
- Freeze the canonical separation between runtime tone and long-term behavior so downstream Phase E work cannot collapse transient presentation into persistent personality drift.
- Preserve the frozen E1 personality architecture and frozen E2 bounded adaptation law while making carry-over, reset, persistence, and no-authority-bleed rules explicit.

C) DEPENDENCY RULE
- E3 consumes E1 and E2 as binding law.
- If E3 wording conflicts with E1 or E2, E1 and E2 win.
- E3 may only separate runtime tone from long-term behavior; it may not redesign persona architecture, adaptive eligibility, or adaptive hard limits.

D) ARCHITECTURAL POSITION
- E3 sits after E1 and E2 and before E4 and E5.
- E3 is a separation/materialization law for personality behavior horizons:
  - `runtime tone horizon`
  - `carry-over candidate horizon`
  - `long-term behavior candidate horizon`
- E3 must preserve the approved stack:
  - E1 defines the canonical layers
  - E2 defines lawful adaptive signals, lawful outputs, forbidden outputs, and deterministic adaptation bounds
  - E3 defines which of those lawful outputs are transient, which may carry, and which may never persist
  - E4 later defines safety/control enforcement on top of the frozen E3 split
  - E5 later verifies and closes the frozen E1-E4 rules

E) E1 / E2 ASSUMPTIONS CONSUMED
- E1 frozen assumptions consumed:
  - `PERSONA_CORE` is stable and authoritative for personality identity
  - `TONE_PRESENTATION_LAYER` is mutable and presentation-only
  - `EMOTIONAL_MODULATION_LAYER` is transient and non-authoritative
  - `MEMORY_FED_PERSONALIZATION_LAYER` may influence delivery but is identity-scoped and memory-bounded
  - `LEARNING_FED_ADAPTATION_LAYER` is advisory and may not directly become personality authority
  - policy, governance, law, identity, and platform are constraint layers only
- E2 frozen assumptions consumed:
  - lawful adaptive inputs are bounded and deterministic
  - only bounded output classes may adapt
  - persona core rewrite, memory truth rewrite, identity truth rewrite, policy/law override, workflow/authority drift, and unbounded self-modification are forbidden
  - deterministic scoring / threshold / decay is required where promotion or carry-over is evaluated
  - runtime-only is the default; persisted adaptation requires bounded later materialization

F) CURRENT TONE / BEHAVIOR SURFACES IN SCOPE
Current Repo Surface → E3 Separation Scope Mapping
| repo surface | current role | authoritative / derived / evidence / decision | E3 relevance | notes / constraints |
| --- | --- | --- | --- | --- |
| `Ph1PersonaRepo` persisted profile snapshot and audit rows | authoritative persona profile persistence | authoritative | anchors stable persona baseline and any lawful coarse preference carry-over already admitted | must not be rewritten by transient tone or single-turn emotion |
| PH1.PERSONA runtime outputs | profile-to-delivery shaping | derived | primary bridge from persona core into runtime tone selection | non-authoritative; may influence delivery only |
| PH1.EMO.CORE emotional signal bundle and guard flags | transient emotional modulation input | derived | defines momentary tone pressure, warmth, assertiveness, distress, and guardrails | tone-only; no execution or authority |
| PH1.EMO.GUIDE style profile and modifiers | deterministic emotional style guidance | derived | shapes runtime tone form and emotional modulation ceiling | reversible, auditable, no-meaning-drift |
| PH1.CONTEXT bundles | situational context shaping | derived | affects runtime framing, explanation depth, acknowledgement detail, and contextual emphasis | advisory only |
| PH1.M memory preference, continuity, suppression, and archive surfaces | identity-scoped memory personalization input | authoritative in memory domain; derived for personality behavior | bounded source of long-term behavior candidates and carry-over eligibility | memory may constrain or nominate; may not rewrite persona core |
| PH1.FEEDBACK signal bundle | explicit correction / satisfaction / preference signal | evidence | bounded nomination source for carry-over or long-term behavior candidates | advisory only; no direct persona mutation |
| PH1.LEARN artifacts | adaptation proposal package | evidence | bounded nomination source for stable behavior candidates | proposal only; no runtime authority |
| PH1.COMP deterministic scoring / threshold math | deterministic quantitative computation | authoritative for computation path only | supplies separation thresholds, persistence eligibility scoring, decay, and reset math | identical inputs must produce identical outputs |
| PH1.POLICY decisions | policy allow/deny and caps | decision | constrains whether style carry-over or persistence is lawful | not a style engine |
| PH1.GOV decisions | governance visibility/decision layer | decision | constrains protected behavior promotion or persistence in governed cases | not a personality writer |
| PH1.LAW final posture | runtime final posture and safety class | decision | can degrade/block/quarantine adaptive effects and carry-over | not a style engine or persona authority |
| identity / voice verification surfaces | identity scope and speaker verification | authoritative for identity | determines whether carry-over may reuse prior identity-scoped behavior candidates | identity truth may not be restyled |
| platform runtime posture | device / environment posture and mechanics | decision / visibility | may constrain runtime tone mechanics and cross-device carry-over admissibility | platform posture may not become personality authority |

G) CANONICAL TONE VS LONG-TERM BEHAVIOR SEPARATION MODEL
- Canonical separation law:
  - `runtime tone` is transient presentation chosen inside a single live interaction horizon
  - `long-term behavior candidate` is a bounded, coarse, evidence-backed preference-like tendency that may later be persisted only through an approved path
  - `persona core` remains outside E3 mutation scope
Runtime Tone vs Long-Term Behavior Matrix
| aspect | runtime tone only / long-term behavior candidate / forbidden persistence | canonical source | carry-over rule | notes |
| --- | --- | --- | --- | --- |
| phrasing style | runtime tone only | PH1.PERSONA runtime outputs plus PH1.EMO.GUIDE modifiers | resets each turn; no raw phrase persistence | coarse preferred register may later be represented only through bounded preference proxies, not phrase history |
| verbosity / pacing | runtime tone only with bounded long-term candidate proxy | PH1.CONTEXT, PH1.M preference, PH1.FEEDBACK, PH1.COMP thresholds | turn-local runtime setting resets; coarse preference band may carry if identity-scoped and repeatedly evidenced | raw per-turn pacing never persists |
| acknowledgement style | runtime tone only with bounded long-term candidate proxy | PH1.PERSONA outputs, PH1.FEEDBACK, PH1.M continuity | acknowledgment wording resets; coarse acknowledgment preference may carry if admitted | no apology/affirmation scripts persist verbatim |
| emotional modulation strength | runtime tone only | PH1.EMO.CORE plus PH1.EMO.GUIDE | resets each turn and session | transient emotion must never become stable profile |
| explanation depth | runtime tone only with bounded long-term candidate proxy | PH1.CONTEXT, PH1.M, PH1.FEEDBACK, PH1.COMP | current-turn depth resets; coarse preferred depth band may carry if repeatedly evidenced | depth persistence must stay within E2 bounds |
| proactive suggestion intensity | runtime tone only with bounded long-term candidate proxy | PH1.PERSONA outputs, PH1.FEEDBACK, PH1.LEARN proposals, PH1.COMP | live intensity resets; stable preference band may carry only through bounded path | may never drift into workflow authority |
| stable preference-like behavior | long-term behavior candidate | identity-scoped memory/preference, repeated feedback, bounded learn proposal | may carry across sessions/devices only after lawful candidate admission | does not mutate persona core |
| persona core traits | forbidden persistence via E3 adaptation path | `Ph1PersonaRepo` authoritative profile | already persistent by persona authority, not by tone carry-over | E3 may not reinterpret or mutate this layer |

H) RUNTIME TONE LAYER
- Runtime tone is produced from:
  - persona profile outputs
  - emotional-state bundle
  - emotional-guide modifiers
  - context bundle
  - lawful identity-scoped preference inputs
  - lawful explicit feedback signals
  - platform posture only where presentation mechanics are affected
- Runtime tone layer properties:
  - per-turn and per-response
  - presentation-only
  - reversible
  - no execution authority
  - no meaning drift
  - no contract drift
Tone Input Source Matrix
| signal | source surface | runtime-only influence or persistence candidate influence | reset rule | notes |
| --- | --- | --- | --- | --- |
| persona profile delivery hints | PH1.PERSONA runtime outputs | runtime-only influence | re-evaluated every turn | stable baseline source, but still runtime delivery guidance |
| emotional signal bundle | PH1.EMO.CORE | runtime-only influence | resets at turn and session boundary | never persisted as stable personality behavior |
| emotional guide modifiers | PH1.EMO.GUIDE | runtime-only influence | recalculated per interaction | may cap or soften modulation but may not create long-term authority |
| context bundle salience | PH1.CONTEXT | runtime-only influence | resets each turn | situational only |
| memory preference hints | PH1.M | persistence candidate influence | memory survives by identity scope; runtime choice still recomputed per turn | influences carry-over candidate evaluation, not direct tone persistence |
| explicit user correction / satisfaction signal | PH1.FEEDBACK | persistence candidate influence | raw feedback event never carries as tone state | may nominate future coarse preference only |
| learn artifact proposal | PH1.LEARN | persistence candidate influence | proposal may be reconsidered later; no direct carry-over | advisory only |
| identity signal | identity / voice surfaces | runtime-only gate and persistence candidate gate | identity verification rechecked per session/device context | governs whether prior candidate behavior may be reused |
| platform posture | platform runtime | runtime-only influence and gate | re-evaluated per device/session | may constrain tone mechanics; may not style identity truth |

I) LONG-TERM BEHAVIOR LAYER
- Long-term behavior in E3 is limited to bounded, coarse, preference-like tendencies.
- Long-term behavior candidate rules:
  - identity-scoped only
  - evidence-backed only
  - deterministic thresholded only
  - policy/governance/law admissible only
  - never allowed to mutate persona core
  - never allowed to persist raw transient tone traces
Long-Term Behavior Candidate Matrix
| candidate behavior | allowed source inputs | persistence eligibility | hard limit | forbidden promotion path | notes |
| --- | --- | --- | --- | --- | --- |
| preferred verbosity band | PH1.M preference signals, repeated PH1.FEEDBACK, bounded PH1.LEARN proposal, PH1.COMP scoring | candidate only | bounded to coarse band, not exact sentence length script | single-turn context, single emotional spike, one-off phrasing | may later persist only through lawful identity-scoped preference form |
| preferred explanation-depth band | PH1.M continuity, repeated explicit feedback, bounded learn proposal | candidate only | bounded by explanation-depth limits from E2 | single-turn explanation outcome | must remain presentation-only |
| preferred acknowledgement style band | repeated explicit feedback, memory continuity | candidate only | coarse style family only; no verbatim acknowledgement persistence | direct phrase copying from prior turns | may never overwrite persona core |
| preferred proactive suggestion ceiling | repeated feedback, memory preference, bounded learn proposal | candidate only | may soften or raise within frozen E2 ceiling only | workflow success/failure logs alone; platform posture alone | must not drift into autonomous task selection |
| preferred emotional modulation ceiling | explicit repeated feedback plus PH1.EMO.GUIDE policy-compatible evidence | candidate only and highly constrained | may cap intensity only; may not encode raw mood | raw PH1.EMO.CORE transient score | safest canonical reading is cap-only, not trait mutation |
| stable preference-like behavior bundle | bounded combination of memory, feedback, and learn proposal | candidate only | identity-scoped, coarse, reversible, auditable | direct write from feedback, raw context, or runtime tone trace | no normalized persisted long-term behavior ledger exists yet |

J) CARRY-OVER, RESET, AND PERSISTENCE RULES
Persistence / Carry-Over / Reset Matrix
| concern | turn boundary rule | session boundary rule | cross-device rule | persisted form allowed or not | notes |
| --- | --- | --- | --- | --- | --- |
| raw phrasing choice | reset | reset | reset | not allowed | wording never carries as stable behavior |
| instantaneous verbosity / pacing choice | reset | reset | reset | not allowed | only coarse preference band may later survive |
| emotional modulation intensity | reset | reset | reset | not allowed | transient emotion never persists |
| explanation depth chosen for one answer | reset | reset | reset | not allowed directly | only coarse preference band may become candidate |
| acknowledgement wording | reset | reset | reset | not allowed | no phrase history persistence |
| proactive suggestion intensity chosen for one turn | reset | reset | reset | not allowed directly | only bounded preference ceiling may become candidate |
| coarse preference-like behavior candidate | may influence later turns if still identity-scoped and threshold-valid | may carry if identity-scoped and not decayed/reset | may carry only through lawful identity-scoped authoritative/preference surfaces | candidate only | no normalized persisted adaptation ledger exists in current slice |
| persona core traits | no change | no change | no change | already authoritative outside E3 | E3 may not mutate this layer |
| raw feedback events | no carry as tone state | no carry as tone state | no carry as tone state | not allowed directly | feedback may only nominate later candidate updates |
| raw context bundle | reset | reset | reset | not allowed | context is situational |
| platform posture | reset when device/runtime changes | re-evaluated | device-specific only | not allowed as personality persistence | platform may gate, never persist as style state |

K) MEMORY / FEEDBACK / LEARNING INTERACTION BOUNDARIES
- Memory, feedback, and learning may nominate long-term behavior candidates only through bounded, deterministic, evidence-backed paths.
- They must never:
  - rewrite persona core
  - persist raw tone traces
  - override identity truth
  - override policy, governance, or law
  - create unbounded self-modification
Disallowed Cross-Layer Mutation Matrix
| forbidden mutation | why forbidden | governing source | must never be treated as | notes |
| --- | --- | --- | --- | --- |
| tone mutating persona core | breaks E1 layer authority | E1 persona-core stability law | lawful personalization | runtime tone is transient only |
| transient emotion mutating stable profile | creates mood-to-trait drift | PH1.EMO.CORE tone-only boundary | personality evolution | emotion may modulate delivery only |
| memory preference mutating authority or identity | memory is not authority | PH1.M deterministic continuity law, Section 07 identity law | identity truth writer | memory may nominate style preference only |
| feedback directly rewriting persona core | feedback is evidence, not authority | PH1.FEEDBACK advisory-only law | persona authoring path | must route through bounded candidate path |
| learning artifact directly rewriting long-term behavior without bounded path | PH1.LEARN is proposal only | PH1.LEARN role, E2 bounds | active personality mutation | learn proposals require separate admission path |
| policy / gov / law being treated as style engines | they are constraint layers | PH1.POLICY / PH1.GOV / PH1.LAW roles | tone generator | they gate and constrain only |

L) GOVERNANCE / LAW / IDENTITY / POLICY / MEMORY CONSTRAINTS
Constraint Matrix
| constraint source | layer affected | hard block or bounded modification | notes |
| --- | --- | --- | --- |
| identity / voice verification | runtime tone and long-term behavior candidate reuse | hard block on cross-user or unverified carry-over | identity mismatch must reset to local runtime-only tone baseline |
| PH1.M suppression / retention / continuity rules | memory-fed personalization and long-term candidate nomination | bounded modification only | suppressed/expired memory may not influence carry-over |
| PH1.POLICY decisions | runtime tone and candidate persistence eligibility | hard block or bounded cap | policy may cap style range and forbid persistence |
| PH1.GOV decisions | governed/policy-sensitive promotion or reuse | bounded modification or hard block | governance remains decision-only, not style author |
| PH1.LAW final posture | live runtime tone effect and carry-over admissibility | hard block / degrade / quarantine / safe-mode | law posture may zero out otherwise lawful adaptive effects |
| platform runtime posture | runtime tone mechanics | bounded modification only | may constrain modality-specific presentation, never personality authority |
| frozen Phase C lifecycle/session law | any persisted or cross-device carry-over path | hard block on hidden writer paths | no parallel persistence path may emerge outside canonical authorities |

M) NO-DRIFT / NO-AUTHORITY-BLEED RULES
- Runtime tone must never become a hidden long-term writer.
- Long-term behavior candidates must never become a hidden persona-core writer.
- Memory, feedback, learning, policy, governance, law, and platform signals may constrain or nominate; they may not silently author personality truth.
- Cross-device/session reuse may consume only identity-scoped, authoritative or lawfully admitted candidate surfaces.
- Deterministic separation rule:
  - if a signal horizon is turn-local or session-local only, classify it as runtime tone
  - if a signal is identity-scoped, repeatedly evidenced, threshold-valid, decay-valid, and policy/governance/law admissible, classify it as a long-term behavior candidate
  - if a signal would mutate persona core, identity truth, memory truth, policy truth, governance truth, or law posture, reject it as forbidden

N) E3 → E4 / E5 FREEZE BOUNDARY
E3 → E4 / E5 Boundary Matrix
| concern | frozen in E3 | deferred to E4 | deferred to E5 | rationale |
| --- | --- | --- | --- | --- |
| canonical runtime tone vs long-term behavior split | yes | no | verify only | E4/E5 may not reopen the split |
| carry-over / reset / persistence horizon rules | yes | no | verify only | these are architectural, not control-plane choices |
| disallowed cross-layer mutation rules | yes | enforce only | verify only | E4 may add controls, not reinterpret the prohibition |
| constraint sources and no-authority-bleed law | yes | enforcement / protected gating only | verify only | E4 may operationalize controls but not change who constrains what |
| runtime-vs-persisted candidate classification | yes | safety/control application only | verify only | E4 may add law/policy memory controls, not expand persistence classes |
| tone-vs-long-term materialization details | no | yes, only within frozen E3 split | verify only | E4 may add control-path integration around materialization already separated here |
| verification / docs / evidence closure | no | no | yes | E5 owns closure only |

O) COMPLETION CRITERIA
- E3 is complete when:
  - runtime tone is frozen as transient presentation-only behavior
  - long-term behavior candidate scope is frozen as bounded, coarse, identity-scoped preference-like carry-over only
  - turn/session/device reset and carry-over rules are frozen explicitly
  - forbidden cross-layer mutations are frozen explicitly
  - governance, law, identity, policy, memory, and platform constraints on separation are frozen explicitly
  - repo-truth gaps are called out explicitly without inventing new persistence or mutation architecture
  - E4 and E5 boundaries are frozen explicitly enough that downstream phases cannot reinterpret E3
