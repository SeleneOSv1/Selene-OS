PHASE E1 — PERSONALITY ARCHITECTURE REVIEW

A) REPO TRUTH CHECK
- `git status --short`: empty
- current branch: `main`
- repo-truth snapshot HEAD at review start: `11132da4df61c0c996b7da8b60e1f56b0f408a71`
- snapshot semantics: this value records the repo state reviewed at the start of this pass; it is not required to equal the later commit hash that records this document update.
- exact files reviewed:
  - `docs/CORE_ARCHITECTURE.md`
  - `docs/SELENE_BUILD_EXECUTION_ORDER.md`
  - `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`
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
  - `crates/selene_kernel_contracts/src/ph1persona.rs`
  - `crates/selene_kernel_contracts/src/ph1e.rs`
  - `crates/selene_kernel_contracts/src/ph1emocore.rs`
  - `crates/selene_kernel_contracts/src/ph1emoguide.rs`
  - `crates/selene_kernel_contracts/src/ph1context.rs`
  - `crates/selene_kernel_contracts/src/ph1m.rs`
  - `crates/selene_kernel_contracts/src/ph1learn.rs`
  - `crates/selene_kernel_contracts/src/ph1policy.rs`
  - `crates/selene_kernel_contracts/src/runtime_governance.rs`
  - `crates/selene_kernel_contracts/src/runtime_law.rs`
  - `crates/selene_storage/src/repo.rs`
  - `crates/selene_storage/src/lib.rs`

B) PURPOSE
- Freeze the canonical personality architecture baseline for Selene without inventing new architecture.
- Separate persona, tone, emotional modulation, memory-fed personalization, and learning-fed adaptation so downstream phases cannot collapse them into one mutable blob.
- Lock authoritative vs derived personality surfaces and prevent governance, law, memory, identity, or learning participants from becoming alternate personality authority.
- Define what is stable, what is mutable, what is derived, and what must remain bounded by policy, governance, law, identity, and memory constraints.

C) DEPENDENCY RULE
- E1 consumes the already approved architecture in `CORE_ARCHITECTURE`, `SELENE_BUILD_EXECUTION_ORDER`, `SELENE_AUTHORITATIVE_ENGINE_INVENTORY`, frozen Phase C authority/lifecycle law, and frozen Phase D authority/consistency/governance-law baselines.
- E1 must not redesign memory authority, session consistency, protected completion, governance posture, or runtime-law posture already frozen in earlier phases.
- Downstream Phase E work may materialize, constrain, or verify this model, but may not reinterpret the canonical layer split, authority split, or mutability law frozen here.

D) ARCHITECTURAL POSITION
- Personality lives in the probabilistic/presentation layer and is bounded to communication style, delivery, tone shaping, and adaptive presentation guidance.
- Personality is not deterministic execution authority.
- Identity, memory, policy, governance, and runtime law may constrain personality behavior, but they do not become personality generators themselves.
- Memory may provide identity-scoped personalization inputs.
- Learning may propose adaptation artifacts.
- Governance and runtime law may allow, degrade, block, quarantine, or safe-mode personality-sensitive behavior where protected paths require it.
- Canonical deterministic authority remains outside personality:
  - access control
  - simulation eligibility
  - mutation
  - ledger writes
  - policy enforcement
  - governance decisioning
  - runtime-law final posture

E) CURRENT REPO SURFACES IN SCOPE
Current Repo Surface → E1 Personality Scope Mapping
| repo surface | current role | authoritative / derived / evidence / decision | personality relevance | notes / constraints |
| --- | --- | --- | --- | --- |
| `PH1.PERSONA` runtime contract + profile build/validate responses | identity-verified style/tone/delivery profile builder | derived | persona-core and delivery-profile input | advisory only; no execution authority; no direct runtime writes in current slice |
| `Ph1PersonaRepo` persisted profile snapshot/audit rows | deterministic persona snapshot persistence path | authoritative within persisted persona-profile storage slice | persona persistence and traceability | runtime slice still treats PH1.PERSONA as no direct writer; storage path is through deterministic repo layer |
| `PH1.EMO.CORE` runtime surface | emotional snapshot/profile core | derived | emotional modulation input | advisory only; tone-only and no-execution-authority constraints are hard-required |
| `PH1.EMO.GUIDE` runtime surface | style/tone guidance | derived | pre-response tone modulation | advisory only; cannot override meaning or authority |
| `PH1.CONTEXT` runtime surface | bounded context composition | derived | presentation-context shaping | no execution authority; may shape what tone/presentation sees, not what authority decides |
| `PH1.M` memory ledgers/current/preferences/continuity | identity-scoped memory and preference truth | authoritative within memory domain | memory-fed personalization input | continuity and preference truth only; never execution authority |
| `PH1.LEARN` adaptation package builder | advisory learning package generation | derived | learning-fed adaptation candidate source | no activation path; package proposals only |
| `PH1.POLICY` policy decision surfaces | deterministic policy decisioning | decision | policy constraint on personality behavior | authoritative for policy decisions only; never personality author |
| `PH1.GOV` governance bundles/state | governance visibility and decision layer | decision | governed personality-sensitive action constraints | visibility/decision only; not a personality writer |
| `PH1.LAW` law posture/state | final runtime-law posture | decision | final posture on personality-sensitive actions | final response posture only; not a personality writer |
| identity/voice verification surfaces (`Section 07`, voice ID contracts) | entry gate into identity-scoped behavior | authoritative within identity domain | binds personality to verified identity scope | if identity fails, identity-scoped personality behavior must fail closed |
| platform runtime posture (`Section 08`, `PH1.OS`) | runtime/device normalization and posture | derived / decision | constrains where personality behavior is admissible | normalization and orchestration only; not personality authority |
| numeric / consensus surfaces (`Section 10`) | deterministic ranking/scoring substrate where later needed | derived / decision | only relevant if later bounded ranking is introduced | no current repo-truth personality authority lives here |

F) CANONICAL PERSONALITY ARCHITECTURE MODEL
- Canonical personality model has five bounded layers plus one constraint layer:
  - `PERSONA_CORE`
  - `TONE_PRESENTATION_LAYER`
  - `EMOTIONAL_MODULATION_LAYER`
  - `MEMORY_FED_PERSONALIZATION_LAYER`
  - `LEARNING_FED_ADAPTATION_LAYER`
  - `CONSTRAINT_LAYER`
- Canonical reading:
  - `PERSONA_CORE` is the relatively stable identity-scoped presentation profile used to select style and delivery.
  - `TONE_PRESENTATION_LAYER` is the immediate response-shaping layer that materializes how the system sounds in a specific turn.
  - `EMOTIONAL_MODULATION_LAYER` is bounded, transient, tone-only modulation fed by emotional state/snapshot logic.
  - `MEMORY_FED_PERSONALIZATION_LAYER` injects identity-scoped continuity and preference truth into presentation decisions.
  - `LEARNING_FED_ADAPTATION_LAYER` proposes longer-term adaptation candidates but does not self-activate.
  - `CONSTRAINT_LAYER` contains policy, governance, runtime-law, identity, and memory safety limits that personality behavior must obey.
- Personality must never collapse into one mutable surface.
- Personality must never outrank:
  - identity truth
  - policy decisions
  - governance decisions
  - runtime-law posture
  - deterministic execution gates

G) PERSONA / TONE / EMOTION / MEMORY / LEARNING BOUNDARIES
Personality Layer Boundary Matrix
| concern | authoritative source | derived / visibility surfaces | mutable or fixed | notes |
| --- | --- | --- | --- | --- |
| persona core | persisted PH1.PERSONA snapshot path plus architecture-level identity-scoped personality classification rule | PH1.PERSONA runtime outputs, PH1.X presentation selection, TTS delivery usage | relatively stable; changed only through deterministic persisted profile evolution | governs delivery/style profile, not meaning or execution |
| tone / presentation layer | none as standalone authority; derived from persona core plus current constraints | PH1.PERSONA outputs, PH1.EMO.GUIDE outputs, PH1.X/TTS runtime presentation path | mutable per interaction | immediate presentation only; must remain reversible and tone-only |
| emotional modulation | PH1.EMO.CORE snapshot/tone guidance within its advisory domain | PH1.EMO.GUIDE style hints, optional persona/context consumption | mutable and transient | advisory only; cannot grant authority or alter meaning |
| memory-fed personalization | PH1.M identity-scoped memory truth, preferences, continuity, suppression, retention posture | PH1.CONTEXT bundles, PH1.PERSONA preference snapshot inputs, PH1.EMO.CORE bounded references | mutable as authoritative memory truth changes | memory informs personalization but never becomes persona authority wholesale |
| learning-fed adaptation | none as direct runtime authority; PH1.LEARN produces candidate packages only | package refs routed toward PH1.PERSONA or other targets | mutable only when future governed activation path admits it | no direct activation path exists in current repo truth |
| policy / law constraints | PH1.POLICY, PH1.GOV, PH1.LAW, identity verification, memory privacy controls | runtime visibility states and decision bundles | mutable only through their own authoritative domains | constraints may bound or block personality behavior; they do not author personality style themselves |

H) AUTHORITATIVE VS DERIVED PERSONALITY SURFACES
Authoritative vs Derived Personality Surface Matrix
| surface | authoritative or derived | current role | must never be treated as | notes |
| --- | --- | --- | --- | --- |
| `Ph1PersonaRepo` persisted profile snapshot row | authoritative | persisted persona-profile storage truth | access authority or runtime-law authority | authority is limited to persona snapshot persistence/history |
| `PH1.PERSONA` runtime build/validate output | derived | persona style/delivery advisory profile | execution authority or direct storage truth | runtime slice says no direct table writes |
| `PH1.EMO.CORE` outputs | derived | emotional state/tone continuity advisory output | personality authority or execution gate | tone-only, no-meaning-drift, no-execution-authority must remain true |
| `PH1.EMO.GUIDE` outputs | derived | style/tone guidance | meaning author or runtime authority | bounded style hints only |
| `PH1.CONTEXT` bundles | derived | context shaping and bounded fusion | long-term personality authority | advisory composition only |
| `PH1.M` memory preferences/continuity truth | authoritative within memory domain | memory-fed personalization source | persona-core author by itself | continuity truth may inform personality but not replace persona layer |
| `PH1.LEARN` artifact packages | derived | adaptation candidate source | active personality mutation by default | no activation path in current repo truth |
| `PH1.POLICY` outputs | authoritative decision | policy constraints | personality author | constrains behavior only |
| `PH1.GOV` outputs | authoritative decision | governance decision/visibility | storage writer for personality state | visibility and decision only |
| `PH1.LAW` outputs | authoritative final posture | final runtime-law posture | personality author or storage writer | final posture only |
| identity verification surfaces | authoritative within identity domain | identity gate for identity-scoped personality | tone/presentation output | identity determines scope, not style |
| platform/runtime posture | derived / decision | device/runtime admissibility signal | personality source of truth | normalization only |

I) MUTABILITY, ADAPTATION, AND LONG-TERM STABILITY LAW
- Stable versus mutable law:
  - persona core is stable by default
  - tone is mutable per interaction
  - emotional modulation is mutable and transient
  - memory-injected preference is mutable only as memory truth changes
  - long-term adaptation is not current runtime truth and must remain explicit future materialization work
- Long-term adaptive personality balancing is not implemented in current repo truth and must not be inferred into live behavior.
- Mutability must remain domain-bounded:
  - memory truth may change memory-fed personalization inputs
  - learning may produce candidate packages
  - policy/governance/law may constrain personality behavior
  - none of those may silently rewrite persona core in-place without a deterministic admitted path

Mutability / Adaptation Matrix
| aspect | current repo truth | allowed to change when | forbidden change mode | downstream phase impact |
| --- | --- | --- | --- | --- |
| persona core | identity-scoped classification/profile artifacts exist; persisted persona snapshot path exists | only through deterministic persisted profile evolution with evidence-backed signals | silent per-turn mutation or device-local mutation | E2/E3 may refine bounded adaptation/materialization, not authority |
| tone | current repo already allows tone-only style shaping | every interaction, within constraints | meaning drift or execution-impacting change | E3 may wire tone/materialization details |
| emotional state | PH1.EMO.CORE and PH1.EMO.GUIDE are optional advisory surfaces | per interaction or bounded continuity window | authority-affecting or irreversible meaning change | E3/E4 may refine controls, not authority split |
| memory-injected preference | PH1.M owns preference/continuity truth | when authoritative memory truth changes lawfully | cross-user bleed, suppressed-memory leak, or direct persona overwrite | E2/E4 may refine bounded use and controls |
| long-term adaptation | current repo says advanced adaptive personality balancing is unimplemented | only after explicit future governed activation design | inferring current live adaptation from learn/emotion output | E2 owns bounded adaptive behavior design |
| policy-gated behavior | policy/governance/law already constrain behavior | only when authoritative decision posture changes | treating policy/law as personality style authors | E4 owns constraint/control design details |

J) GOVERNANCE / LAW / MEMORY / IDENTITY / POLICY CONSTRAINTS
Governance / Law / Memory / Identity Constraint Matrix
| constraint source | scope of effect | personality impact | must never become | notes |
| --- | --- | --- | --- | --- |
| identity verification (`Section 07`, voice/identity contracts) | entry into identity-scoped personality behavior | blocks identity-scoped personalization if verification is missing or failed | a style/tone generator | identity gates scope, not presentation content |
| PH1.M memory privacy/suppression/retention controls | memory-fed personalization and continuity injection | may suppress or remove personality-relevant continuity inputs | a persona-core rewrite engine | memory safety wins over personalization continuity |
| PH1.POLICY | prompt, content, and behavior constraints | may constrain or refuse presentation modes | persona author or emotion engine | authoritative decisions only |
| PH1.GOV | governed risk/visibility paths | may require visibility, acceptance, or refusal before protected adaptive behavior | storage writer for personality state | decision layer only |
| PH1.LAW | final runtime-law posture | may allow, degrade, block, quarantine, or safe-mode personality-sensitive behavior | persona generator | final posture only |
| PH1.OS / platform runtime posture | device/runtime admissibility and normalization | may narrow allowed delivery behavior for runtime posture reasons | personality authority | normalization and orchestration only |
| identity-scoped policy context refs in memory/context surfaces | bounded contextual constraint injection | personality behavior must respect policy context | alternate policy engine | contextual inputs remain subordinate to policy decisions |

K) CURRENT CONFLICTS / GAPS
- Repo-truth conflict 1:
  - `CORE_ARCHITECTURE` defines personality classification as `Passive`, `Domineering`, or `Undetermined`.
  - PH1.PERSONA contracts center around style profile, delivery policy, and preference snapshot refs.
  - PH1.EMO.CORE exposes `personality_type` and `personality_lock_status`.
  - Safest canonical reading: these are not competing authorities; the classification/profile/core surfaces are different slices of one bounded presentation architecture and none may influence deterministic authority.
- Repo-truth conflict 2:
  - PH1.PERSONA runtime wiring says no direct runtime writes.
  - Storage repo truth includes persisted persona profile commit/audit rows.
  - Safest canonical reading: PH1.PERSONA runtime remains no direct writer, while deterministic repo/storage layers own persisted profile materialization.
- Repo-truth gap 3:
  - current repo truth does not yet normalize one canonical mapping from memory preferences/continuity to persona snapshot evolution.
  - Safest canonical reading: memory is an input source only; direct memory-to-persona mutation is not assumed.
- Repo-truth gap 4:
  - long-term adaptive personality balancing remains unimplemented.
  - Safest canonical reading: no live runtime may claim long-term adaptive personality authority today.
- Repo-truth gap 5:
  - numeric/consensus surfaces may later support scoring or ranking, but no canonical current personality scoring authority exists there.
  - Safest canonical reading: Section 10 is only a future bounded relevance area, not current personality authority.

Current Repo Surface to Canonical E1 Mapping
| current surface | canonical E1 role | authority class | downstream phase most affected | notes |
| --- | --- | --- | --- | --- |
| `PH1.PERSONA` runtime contracts | persona-core advisory builder/validator | derived | E3 | no direct writes in runtime slice |
| `Ph1PersonaRepo` persisted rows | persona-core persistence/history path | authoritative within persona persistence | E3 | deterministic repo-owned storage path |
| `PH1.EMO.CORE` | emotional modulation core | derived | E3 | snapshot/tone continuity only |
| `PH1.EMO.GUIDE` | tone/presentation style hint source | derived | E3 | bounded modifiers only |
| `PH1.CONTEXT` | memory/context presentation composition | derived | E3 | no execution authority |
| `PH1.M` preferences/continuity | memory-fed personalization input truth | authoritative within memory domain | E2 / E4 | continuity and privacy controls are memory-owned |
| `PH1.LEARN` artifacts | learning-fed adaptation candidate source | derived | E2 | advisory package builder only |
| `PH1.POLICY` | policy constraint source | decision | E4 | policy may constrain behavior but not author personality |
| `PH1.GOV` | governance visibility/decision source | decision | E4 | governs risky adaptation/behavior classes |
| `PH1.LAW` | final runtime-law posture source | decision | E4 | applies final posture to personality-sensitive paths |
| identity/voice verification | identity-scope gate | authoritative within identity domain | E4 | personality must fail closed when identity scope is missing |
| platform runtime posture | runtime admissibility constraint | derived / decision | E4 | delivery may be constrained by posture |

L) E1 → E2 / E3 / E4 / E5 FREEZE BOUNDARY
E1 → E2 / E3 / E4 / E5 Boundary Matrix
| concern | frozen in E1 | deferred to E2 | deferred to E3 | deferred to E4 | deferred to E5 | rationale |
| --- | --- | --- | --- | --- | --- | --- |
| persona vs tone vs emotion vs memory vs learning separation | yes | no | no | no | no | canonical layer split must not be reopened |
| bounded adaptive behavior design | authority split and allowed source domains only | yes | no | constraint overlays only | verify only | E2 may design bounded adaptation behavior without reinterpreting layer authority |
| tone vs long-term behavior materialization | canonical distinction only | no | yes | constraint overlays only | verify only | E3 may wire runtime separation/materialization |
| memory / policy / governance / law control details | constraint-source identity only | bounded adaptive dependencies only | runtime materialization dependencies only | yes | verify only | E4 owns concrete control/alignment design |
| persona persistence schema/wiring materialization | persona persistence is authoritative within its slice | bounded adaptive consumers only | yes | no | verify only | E3 may materialize schema/wiring details without changing authority |
| long-term adaptation activation path | current repo says not active | yes | supporting runtime materialization only | governed constraints only | verify only | future adaptation must stay bounded and governed |
| closure verification and evidence pack | freeze boundary only | no | no | no | yes | E5 owns tests/docs/verification closure |

M) COMPLETION CRITERIA
- E1 is complete only if:
  - persona, tone, emotional modulation, memory-fed personalization, and learning-fed adaptation are canonically separated
  - authoritative vs derived personality surfaces are explicitly frozen
  - mutability and long-term stability law are explicitly frozen
  - governance, law, memory, identity, and policy constraints are explicitly frozen
  - current repo-truth conflicts and gaps are explicitly bounded
  - E1 → E2 / E3 / E4 / E5 boundaries are explicit enough that downstream phases cannot reinterpret E1
- If a later phase needs a different authority split, different stable/mutable split, or different constraint model, E1 must be reopened explicitly rather than drifted silently.
