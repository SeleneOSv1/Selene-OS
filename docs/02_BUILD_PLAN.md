# Selene OS Build Plan

Related docs:
- One-page engine inventory: `docs/07_ENGINE_REGISTRY.md`.
- Runtime wiring overview: `docs/06_ENGINE_MAP.md`.
- Behavioral laws/spec: `docs/05_OS_CONSTITUTION.md`.
- Ordered design lock sequence: `docs/11_DESIGN_LOCK_SEQUENCE.md`.

## Design Lock Order (before broad runtime wiring)
- Follow `docs/11_DESIGN_LOCK_SEQUENCE.md` in strict order.
- Do not start broad runtime wiring until items 1-9 are locked.

## Section P0: Execution-Grade Layer (must be built first)
- Freeze kernel contracts + validators: goal: establish immutable runtime envelopes and strict validation; required artifacts: `selene_kernel_contracts` types, validators, version policy; acceptance bar: all runtime boundaries reject unknown/invalid envelopes deterministically.
- Enforce mediation (no direct engine calls): goal: make Selene OS the only orchestrator; required artifacts: orchestrator dispatch boundary, enforcement tests, static call graph checks; acceptance bar: engine-to-engine dispatch attempts are blocked and test-proven.
- Idempotency + outbox + retries + dedupe: goal: guarantee no duplicate side effects across retries/restarts; required artifacts: idempotency key spec, outbox schema, retry scheduler, dedupe constraints; acceptance bar: repeated requests with same key produce one logical effect.
- DB role hardening + break-glass: goal: prevent manual mutation of protected ledgers; required artifacts: Postgres role model, grants matrix, audited break-glass runbook; acceptance bar: forbidden writes are blocked by role policy and break-glass usage is auditable.
- Observability + replay tooling: goal: reconstruct every decision path by correlation ID; required artifacts: canonical audit schema, correlation propagation, replay CLI; acceptance bar: deterministic replay output for the same correlation ID is identical across runs.
- Benchmark + chaos harness: goal: validate reliability under adverse conditions; required artifacts: benchmark corpus, fault injection scenarios, pass/fail thresholds; acceptance bar: SLO and failure-recovery criteria pass under defined chaos profiles.

## Section P0.K: PH1.K Voice Runtime I/O (foundation for all voice)
- K0 (interface lock): goal: lock the **versioned PH1.K contracts** (streams, frames, pre-roll, VAD, device state/health, timing stats, interruption candidates, degradation flags) and reason-code space; acceptance bar: upstream engines only consume PH1.K events/refs and never infer semantics from raw audio.
- K0A (stream + state semantics): goal: lock deterministic stream semantics (monotonic `seq_no`, monotonic `t_capture`, fixed frame size per stream, explicit gap/degradation signals) and the explicit state machine + reason-coded transition events; acceptance bar: replay/determinism tests prove ordering, gap signaling, and state transitions.
- K1 (deterministic harness): goal: build record/replay + fault injection for PH1.K (device hot-plug, permission loss, stream gaps, underruns/overruns); acceptance bar: core PH1.K behavior is provable offline and repeatable without manual listening.
- K2 (real full-duplex loop): goal: implement mic capture + speaker playback at fixed 10ms/20ms frames with capture that never blocks (ring buffers, bounded queues); acceptance bar: sustained full-duplex run with no frame drops under normal load.
- K3 (canonical format boundary): goal: enforce one internal audio format (sample rate/channels/sample format) and convert at the boundary; acceptance bar: downstream wake/STT/VAD see stable, deterministic audio frames regardless of device quirks.
- K4 (pre-roll always-on): goal: maintain a rolling processed pre-roll buffer (~1.0–1.5s); acceptance bar: wake/speech start never clips first syllables.
- K5 (timing + jitter + drift): goal: adaptive jitter buffers + drift measurement + correction; acceptance bar: long-running sessions keep stable latency; timing stats are emitted and match observed behavior.
- K6 (DSP pipeline): goal: AEC + NS + AGC as the default processed stream; acceptance bar: echo/room noise is suppressed enough that wake/interrupt are reliable in common environments.
- K7 (echo-safe contract with PH1.TTS): goal: treat `tts_playback_active` + echo-safe gating as first-class (PH1.TTS tags playback boundaries; PH1.K uses it); acceptance bar: Selene does not self-trigger wake/interrupt during its own speech.
- K8 (device policy OS-grade): goal: deterministic selection order + failover + hot-plug handling with anti-thrash cooldowns and reason-coded events; acceptance bar: device changes are boring (no flapping) and always observable upstream.
- K9 (interruption detection): goal: phrase-first interruption candidates during TTS with strict multi-gate confidence (VAD + echo-safe + phrase + optional near-field) and stable phrase IDs; acceptance bar: Selene only stops speaking on high-confidence intent to interrupt (no “random noise” stops).
- K9A (barge-in loop integration): goal: wire `interrupt_candidate` through the full barge-in chain (PH1.K emits → PH1.X issues `tts_cancel` and maintains Resume Buffer rules → PH1.TTS stops immediately); acceptance bar: deterministic tests prove interruption stops speech instantly and interrupted responses are never treated as delivered.
- K10 (telemetry budgets): goal: publish measurable latency/quality signals (p50/p95/p99 capture→frame latency, VAD latency, buffer depth, underruns/overruns, AEC stability); acceptance bar: “feels human” targets are enforced by data, not vibes.
- K11 (privacy defaults): goal: raw audio is policy-gated/off by default, with explicit consent + TTL when enabled; acceptance bar: by default Selene stores only derived events/metrics/hashes needed for audit and replay.

## Section P0.C: PH1.C STT Router + Quality Gate (trusted transcript or fail-closed)
- C0 (contract lock): goal: lock PH1.C request/response contracts + validators (transcript_ok/reject only; no provider leakage); acceptance bar: identical audio + policy + package versions produce identical transcript_ok or identical reject + reason_code.
- C1 (policy routing gate): goal: enforce enterprise locality/privacy constraints (on-device/on-prem/cloud allowed) with route_class auditability; acceptance bar: local_only policies never silently fall back to cloud; violations fail closed with a deterministic reason.
- C2 (quality scoring 3-part): goal: coverage + confidence + plausibility scoring stays deterministic and bounded; acceptance bar: broken/garbled audio never reaches PH1.NLP; retries happen only within strict budgets.
- C3 (speaker focus integration): goal: optionally prefer primary-speaker segments to reduce TV/background pollution; acceptance bar: background speech does not blend into the transcript; reject when separation is not reliable.
- C4 (vocabulary packs): goal: versioned tenant/user vocabulary packs improve names/terms without rewriting meaning; acceptance bar: enterprise acronyms and product names are preserved deterministically; user packs require identity OK and policy allow.
- C5 (uncertain spans): goal: provide bounded provider-agnostic uncertain_spans for targeted clarification without guessing; acceptance bar: uncertain spans never become guessed words; they only enable one precise question later.
- C6 (quota/cost guardrails): goal: deterministic throttles and budget caps; acceptance bar: quota exceed fails closed with safe fallback path (example: switch to text).
- C7 (offline regression suite): goal: golden audio corpus prevents regressions across accents/noise/environments; acceptance bar: new packages/providers can’t ship if any required bucket regresses.

## Section P0.UI: Text Modality + History (voice-first, not voice-only)
- UI0 (text transcript contract): goal: typed input enters the same pipeline as voice without forking logic; required artifacts: a `transcript_ok`-equivalent contract for text input (same fields, same evidence discipline); acceptance bar: OS.5 gate order is identical for voice vs text except the input channel.
- UI1 (conversation history ledger): goal: make scrollback + recall real and durable; required artifacts: append-only `conversation_ledger` in PH1.F, OS writes for every user turn and every Selene response_text, audit references (`CONVERSATION_TURN_STORED`); acceptance bar: history is reconstructible by correlation_id and never silently mutates.
- UI2 (chat surface wiring): goal: a simple ChatGPT-like chat surface exists as a first-class interface; required artifacts: typed input path, text rendering path (always render Selene output as text even when spoken), session-close clears screen but archives; acceptance bar: users can scroll full history and explicitly recall archived sessions.
- UI3 (phone app delivery via broadcast): goal: "display it on my phone" delivers to the Selene phone app UI (not SMS) safely; required artifacts: BROADCAST side effect type + idempotent outbox integration + Broadcast engine skeleton; acceptance bar: access/authority + simulation gates enforced; no duplicate deliveries across retries/restarts.

## Section P0.WRITE: PH1.WRITE Professional Writing & Formatting (presentation only)
- WRITE0 (contract lock): goal: lock PH1.WRITE boundaries (presentation only; no meaning drift) and critical-token preservation rules; acceptance bar: formatting can never change names/numbers/dates/amounts or weaken refusals/policy text.
- WRITE1 (single truth for voice+text): goal: ensure the UI and PH1.TTS speak/render the exact same formatted_text; acceptance bar: no "screen says one thing, voice says another" because both consume PH1.WRITE output.
- WRITE2 (safe fallback): goal: if PH1.WRITE cannot produce a safe rewrite deterministically, it returns the original response_text unchanged; acceptance bar: failures never cause unsafe or different wording.
- WRITE3 (acceptance tests): goal: implement AT-WRITE-01 and AT-WRITE-02 from the Constitution; acceptance bar: tests prove critical tokens and refusals are preserved.

## Section P1: Reliability
- Runtime resilience hardening: goal: stable operation through intermittent provider/device faults; required artifacts: fallback policies, timeout budgets, bounded retries; acceptance bar: known transient failures recover without unsafe execution.
- Contract evolution safety: goal: support controlled schema upgrades; required artifacts: N/N-1 compatibility rules, migration tests, deprecation policy; acceptance bar: old supported clients continue passing contracts without drift.
- Clarification determinism: goal: guarantee consistent missing-field question selection; required artifacts: blocking-field priority rules, test fixtures; acceptance bar: same inputs always produce same clarify question.
- Tool provenance integrity: goal: keep read-only results traceable and conflict-aware; required artifacts: provenance metadata schema, conflict flags, freshness policy; acceptance bar: tool outputs always include structured provenance and conflict handling.

## Section P2: World-Class
- Latency and quality envelopes: goal: enforce p50/p95/p99 interaction budgets; required artifacts: SLO definitions, budget guards, perf dashboards; acceptance bar: production workloads stay within agreed latency/error envelopes.
- Cost guardrails: goal: keep per-user/day spend predictable; required artifacts: budget policy, per-turn accounting, throttling rules; acceptance bar: budget overruns are prevented or fail closed deterministically.
- Multi-tenant hard isolation: goal: ensure tenant-safe execution and storage boundaries; required artifacts: tenant propagation contract, isolation tests, policy gates; acceptance bar: cross-tenant access attempts fail by default with audit proof.
- Governance controls: goal: provide rapid safe-mode and kill-switch operations; required artifacts: governance simulations, operator controls, incident playbooks; acceptance bar: incident controls execute deterministically and are fully auditable.

## Section P2.LA: Learning & Adaptation Layer (LA) (New Engines)
- LA0 (spec lock): goal: freeze LA.* engine boundaries + acceptance tests in `docs/05_OS_CONSTITUTION.md` and list them in `docs/06_ENGINE_MAP.md`; acceptance bar: no conflicts with "no guessing / no silent authority / no execution without access + simulation".
- LA1 (feedback first): goal: implement PH1.FEEDBACK + PH1.LEARN packaging skeleton so Selene can improve deterministically; acceptance bar: correction/reject/fallback events emit FeedbackEvent and produce versioned, rollbackable artifacts.
- LA2 (tenant/user vocab + pronunciation): goal: implement PH1.KNOW + PH1.PRON packs and wire into PH1.C/PH1.TTS; acceptance bar: names/terms improve without meaning drift; tenant isolation is provable.
- LA3 (provider arbitration): goal: implement PH1.PAE deterministic provider plans (SHADOW -> ASSIST -> LEAD); acceptance bar: no silent promotions; deterministic promote/demote with audit reason codes.
- LA4 (persona + delivery policy): goal: implement PH1.PERSONA and apply tone/delivery hints only when identity is OK; acceptance bar: unknown speaker -> no persona; persona never changes intent/truth.
- LA5 (semantic repair + clarification minimization): goal: implement PH1.SRL + PH1.PRUNE; acceptance bar: repair introduces no new facts; clarify stays one question; ambiguity forces clarify.
- LA6 (endpointing + language): goal: implement PH1.ENDPOINT + PH1.LANG; acceptance bar: faster turns with fewer clipped utterances; no forced translation; code-switch preserved.
- LA7 (context + cache): goal: implement PH1.CONTEXT + PH1.CACHE as bounded advisory helpers; acceptance bar: cache/context never bypass gates; all context is evidence-backed.
- LA8 (offline optimizers): goal: implement PH1.PATTERN + PH1.RLL as offline-only artifact generators; acceptance bar: outputs are artifacts only and require approval to activate; never a runtime controller.
