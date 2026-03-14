PHASE H1 — BUILD METHOD AND IMPLEMENTATION SEQUENCE

A) PURPOSE
- H1 is the formal implementation-readiness and sequencing plan derived from frozen design law.
- H1 is planning only. It does not authorize code changes, build commands, or execution task lists.
- H1 does not reopen frozen Phase A-G design law and does not modify any frozen phase document.

B) FROZEN LAW INPUTS
- H1 consumes frozen design law from:
  - Phase `A1-A6`
  - Phase `B1-B5`
  - Phase `C1-C5`
  - Phase `D1-D5`
  - Phase `E1-E5`
  - Phase `F1-F5`
  - Phase `G1-G2`
- H1 also consumes Build Sections `01-11` as the engineering dependency law for runtime implementation.
- H1 treats the build execution order document, the authoritative coverage table, and the design-lock sequence as gating truth for implementation sequencing.
- H1 therefore inherits these non-negotiable implementation facts:
  - implementation must proceed in dependency order
  - runtime foundation must lead
  - clients are first-class but non-authority
  - partially generalized engines may not be treated as globally complete
  - Apple implementation may not outrun the runtime substrate

C) CURRENT / TARGET / GAP
- `CURRENT`
  - frozen A-G design law exists and is sufficient to start dependency-ordered implementation
  - the design-lock sequence is already locked through minimal runtime wiring
  - the authoritative coverage table still marks `PH1.J`, `PH1.M`, `PH1.OS`, `PH1.COMP`, and `PH1.LAW` as only partially generalized at the architecture-closure level
- `TARGET`
  - implement Selene using `HYBRID_BUILD_AND_DESIGN`
  - start implementation now from the frozen foundation
  - keep closure/generalization work running in parallel where repo truth still says architecture closure is incomplete
- `GAP`
  - the repo does not justify a full feature-first build push
  - the repo also does not justify waiting for every last downstream closure artifact before starting implementation
  - the missing piece is sequencing discipline, not a new architecture phase

D) CHOSEN BUILD METHOD
- The chosen build method is `HYBRID_BUILD_AND_DESIGN`.
- This is the correct method because repo truth says:
  - broad runtime wiring may begin once the design-lock prerequisites are locked
  - implementation must still obey strict dependency order
  - parallel work is allowed only when dependency law is preserved
  - higher-risk and partially generalized areas must not be treated as already complete
- H1 therefore rejects two bad extremes:
  - do not wait for total-system design closure before starting any implementation
  - do not start broad full-stack implementation as if all architecture closure work were already complete
- H1 also captures two specific repo-truth conclusions from readiness review:
  - Session is buildable now, but it must remain inside session-law guardrails.
  - Personality / emotional behavior is not a true blocker, but it must remain tone-only and non-authoritative.

E) strategy comparison matrix
| option | strength | failure mode for Selene | sequencing judgment |
| --- | --- | --- | --- |
| `FINISH_FULL_SYSTEM_DESIGN_FIRST` | reduces pressure on still-partial closure areas | over-constrains the repo because design lock is already sufficient for broad runtime start and the build order already defines safe sequencing | rejected as the primary method |
| `START_BUILD_NOW_FROM_FROZEN_FOUNDATION` | matches kernel-outward implementation law and avoids unnecessary planning drift | becomes unsafe if interpreted as broad feature-first build while partially generalized areas still exist | rejected as the standalone method |
| `HYBRID_BUILD_AND_DESIGN` | starts from frozen foundation while preserving dependency law and parallel closure work | requires strict discipline so later layers and Apple UI do not outrun Sections `01-05` | chosen |

F) build-readiness classification matrix
| area | classification | implementation posture |
| --- | --- | --- |
| Core runtime skeleton | `BUILD_READY_NOW` | must be implemented first as the runtime kernel |
| Session | `BUILD_READY_WITH_TIGHT_GUARDRAILS` | build now, but preserve canonical session law and close-check discipline |
| Ingress + turn pipeline | `BUILD_READY_WITH_TIGHT_GUARDRAILS` | build now through the single canonical execution path only |
| Authority / governance / law / proof | `BUILD_READY_WITH_TIGHT_GUARDRAILS` | build in order, but do not over-claim generalized protected-path completion |
| Persistence + sync | `BUILD_READY_WITH_TIGHT_GUARDRAILS` | build now with strict idempotency, replay, and reconcile discipline |
| Memory engine | `BUILD_READY_WITH_TIGHT_GUARDRAILS` | build after the spine is stable; do not treat memory closure as globally finished |
| Personality / emotional model | `BUILD_READY_WITH_TIGHT_GUARDRAILS` | build later and only as bounded tone/delivery behavior |
| iPhone app | `BUILD_READY_WITH_TIGHT_GUARDRAILS` | design is frozen, but implementation waits for runtime spine stability |
| Mac app | `BUILD_READY_WITH_TIGHT_GUARDRAILS` | design is frozen, but implementation waits for runtime spine stability |
| Platform runtime | `BUILD_READY_WITH_TIGHT_GUARDRAILS` | build in dependency order; do not treat platform closure as generalized complete |
| PH1.COMP | `DESIGN_FIRST_BEFORE_BUILD` | do not broadly adopt until quantitative authority paths are normalized |
| True blocker state | `none currently` | no current repo-truth blocker prevents foundation build start |

G) SEQUENCING LAW
- Build must proceed in Build Section dependency order and must stop when a later slice depends on an uncertified earlier slice.
- Selene must not be built feature-first, UI-first, or app-first.
- Apple implementation must not begin as the leading workstream.
- Apple clients remain downstream session interfaces for the same cloud runtime and may not force local workaround architecture.
- `PH1.J`, `PH1.M`, `PH1.OS`, `PH1.COMP`, and `PH1.LAW` may not be treated as globally complete simply because design depth or partial wiring already exists.
- Session implementation must preserve:
  - cloud-owned session truth
  - canonical `Closed/Open/Active/SoftClosed/Suspended` state law
  - no client-authored attach/recover/close truth
- Personality and emotional implementation must preserve:
  - tone-only effect
  - no meaning drift
  - no execution authority
  - no local persona or emotional authority path

H) slice sequencing matrix
| slice | objective | why this order is lawful | must not violate |
| --- | --- | --- | --- |
| Slice 1 | establish the runtime kernel and canonical session container across Sections `01-02` | the build order requires kernel first and session before deeper execution flow | no alternate runtime substrate, no client-local session truth, no feature-first shortcuts |
| Slice 2 | establish the canonical execution path across Sections `03-05`: ingress, authority, and idempotent persistence | protected execution is not trustworthy until the envelope, authority boundary, and retry/reconcile seam are stable | no alternate ingress family, no simulation bypass, no duplicate side effects |
| Slice 3 | establish bounded downstream runtime surfaces needed by clients across Sections `06-08`, while staying below generalized law closure | clients need stable read-model carriers, but only after the runtime spine is real | no local memory authority, no app-owned system truth, no platform-specific execution fork |

I) parallel design / closure-track matrix
| track kept in parallel | why it stays parallel while implementation starts | required posture |
| --- | --- | --- |
| `PH1.J` generalized protected-path adoption | proof depth exists, but architecture-wide protected adoption is still not globally closed | continue closure/generalization work without blocking Section `01-05` foundation build |
| `PH1.M` architecture closure over retention and governance surfaces | memory design is strong, but the authoritative coverage table still marks closure incomplete | continue memory-closure work while building the earlier spine |
| `PH1.OS` final architecture closure | platform runtime exists, but generalization is not fully closed | keep platform closure work bounded and behind earlier runtime dependency gates |
| `PH1.COMP` quantitative authority normalization | deterministic computation exists, but parallel quantitative authority paths remain | treat as design-first before broad adoption |
| `PH1.LAW` generalized protected-path live wiring | final law engine exists, but global live-path generalization is not yet complete | continue law-generalization and certification work in parallel |
| closure evidence families from `A6/B5/C5/D5/E5` | frozen design law exists, but closure artifacts still define evidence gates and residual-risk discipline | continue evidence, traceability, and closure work while foundation implementation advances |
| bounded session close-behavior refinement | Session is buildable now, but close-confirmation detail remains sensitive | refine without reopening session architecture |
| Apple implementation-readiness refinement | Apple design is frozen, but app sequencing remains gated by runtime readiness | keep Apple on a readiness track until the entry gate is satisfied |

J) Apple implementation entry-gate matrix
| gate item | required posture before Apple implementation begins | result |
| --- | --- | --- |
| runtime kernel stability | Section `01` must be real enough that the app does not invent local execution substrate | mandatory |
| session truth stability | Section `02` must expose stable lawful session state, attach, recover, and suspend posture | mandatory |
| canonical ingress truth | Section `03` must provide the single governed turn path and runtime envelope | mandatory |
| authority boundary stability | Section `04` must be stable enough that Apple surfaces remain first-class but non-authority | mandatory |
| persistence and reconcile stability | Section `05` must be stable enough that Apple does not invent local retry, dedupe, or repair authority | mandatory |
| earliest allowed Apple start | Apple implementation may begin only after the runtime / session / ingress / authority / persistence spine is stable enough to support G1 and G2 as thin lawful session interfaces | earliest lawful start |
| first Apple scope once opened | start with session surface, transcript rendering, bounded history, `resume context`, and separate `System Activity` / `Needs Attention` | allowed |
| forbidden early Apple scope | no app-first protected execution, no local authority caches, no local proof/governance/law behavior, no UI-led runtime redesign | forbidden |

K) risk / containment matrix
| risk area | why it is dangerous now | containment posture |
| --- | --- | --- |
| feature-first implementation drift | it breaks the repo’s kernel-outward dependency law | keep implementation locked to Section order and stop on uncertified dependency |
| Apple UI outrunning runtime truth | it would push local workaround architecture into clients | hold Apple implementation behind the explicit entry gate in `J)` |
| treating partial engines as complete | it creates false closure around `PH1.J`, `PH1.M`, `PH1.OS`, `PH1.COMP`, and `PH1.LAW` | keep those areas on the parallel closure track and do not widen claims |
| session-law drift | local UI or convenience logic could redefine attach, recover, or close semantics | keep session implementation inside canonical session law only |
| personality or emotional authority bleed | tone/adaptation work could expand into truth, workflow, or memory authority | keep personality tone-only, no-drift, and no-execution-authority |
| premature PH1.COMP spread | quantitative authority may fork across engines before normalization | treat PH1.COMP as design-first before broad adoption |
| governance/law optionality | later slices could act as if final governance/law posture is a polish step | preserve governance/law as required completion posture, not optional hardening |

L) READINESS OUTCOME
- H1 records the formal readiness outcome as:
  - build foundation now
  - keep closure/generalization work in parallel
  - do not delay on Session
  - do not delay on bounded Personality / emotional work
  - do delay Apple implementation until the runtime/session/ingress/authority/persistence spine is stable enough
- H1 records no current repo-truth `TRUE_BLOCKER` for starting implementation in controlled form.
- H1 records the strongest near-term design-first constraint as `PH1.COMP` broad adoption and the strongest near-term sequencing constraint as Apple staying behind the runtime spine.

M) PHASE BOUNDARY
- H1 is derived from frozen A-G and Build Sections `01-11`.
- H1 is sequencing law only.
- H1 does not create code tasks, build commands, or implementation tickets.
- H1 does not authorize deviations from dependency order, non-authority client law, or frozen Apple design law.
- The next step after H1 is implementation-readiness execution planning against this sequence, not architecture redesign.
