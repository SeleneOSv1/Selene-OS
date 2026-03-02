# Web Search Plan

Status: Build Plan (Deterministic, Evidence-Grounded)

## 0) Objective

Selene must perform web/news retrieval and return:

- Clean synthesized answer
- Backed by real sources
- With explicit citations
- Fail-closed posture (no invented facts)
- Deterministic execution boundaries

Hard rule: No `SimulationExecutor` involvement for this path. This remains read-only lane behavior.

## 1) Authoritative Architectural Flow

`PH1.X` decides `need web/news/url`  
`-> PH1.SEARCH` (includes merged web intelligence capability)  
`-> Evidence Packet` returned  
`-> PH1.D` synthesizes strictly from Evidence Packet  
`-> PH1.WRITE` formats output  
`-> PH1.J` audits (`query`, providers used, evidence hash, response hash)

Hard rule: `PH1.D` may only use the Evidence Packet as truth input.

## 2) Provider Strategy (Updated)

### Web
- Lead: Brave Web Search
- Fallback: OpenAI `web_search` (policy and constraints are canonical in Section 8)

### News
- Lead: Brave News Search
- Assist (Implemented Now): GDELT for global recall + corroboration

Deterministic merge:

- Deduplicate by `canonical_url`
- Brave order first
- GDELT unique results appended

### URL Fetch + Cite
- In-house implementation
- HTTP fetch
- Text extraction
- Deterministic chunking + hashing
- Citation references to `chunk_id`s

### Images + Video (Implemented Now)
- Add `ImageSearchQuery` and `VideoSearchQuery` tool enums
- Provider: Brave Image Search + Brave Video Search
- Return metadata + URLs only
- No media downloading
- Deterministic ordering
- Citations included

## 3) Evidence Packet Contract

Each retrieval returns an Evidence Packet:

- `query`
- `retrieved_at` (unix ms)
- `provider_runs[]`
- `provider_runs[].provider_id`
- `provider_runs[].endpoint` (`web` / `news` / `images` / `video` / `url_fetch`)
- `provider_runs[].latency_ms`
- `provider_runs[].cache_status`
- `provider_runs[].error` (optional)
- `sources[]`
- `sources[].title`
- `sources[].url`
- `sources[].snippet` (optional)
- `sources[].published_at` (optional)
- `sources[].media_type` (`web` / `news` / `image` / `video` / `document`)
- `sources[].provider_id`
- `sources[].rank`
- `content_chunks[]` (for URL fetch)
- `content_chunks[].chunk_id` (hash)
- `content_chunks[].text_excerpt` (bounded)
- `content_chunks[].source_url`
- `dedup + trust metadata`
- `canonical_url`
- `trust_tier`
- `freshness_score`
- `corroboration_count`
- `contradiction_group_id` (optional)

Hard rule: `PH1.D` cannot use anything outside this packet.

## 4) PH1.D Synthesis Contract (Required)

Inputs:

- User question
- Evidence Packet

Outputs (strict schema):

- `answer_text`
- `bullet_evidence[]` (each item cites 1+ sources)
- `citations[]`
- `uncertainty_flags`
- `reason_codes`

Rules:

- No citation invention
- If insufficient evidence, explicitly state it
- If conflicting evidence, flag the conflict
- PH1.D must perform deterministic clarity cleanup (grammar/coherence/conciseness) while preserving evidence meaning.
- PH1.D may improve wording clarity, but must not introduce new facts, entities, dates, amounts, or claims.

## 4.1 PH1.WRITE Structured Output Rules

PH1.WRITE is mandatory for final user-facing text rendering in this plan; raw PH1.D synthesis text is not returned directly.
Text and voice must share one canonical `formatted_text` output from PH1.WRITE. If voice is enabled, PH1.TTS speaks this same `formatted_text` (no separate semantic rewrite path).

For text output, PH1.WRITE must format responses with:

- Clear headings
- Bullet points
- Explicit citations
- Professional writing quality (clear grammar, concise structure, deterministic ordering)

Example format:

1. Direct Answer  
Short, clear answer (2-4 sentences max).
2. Evidence  
Bullet claim (cites >=1 source or `chunk_id`)  
Bullet claim (cites >=1 source or `chunk_id`)
3. Citations  
URL 1  
URL 2

For voice output:

- Use short, direct sentences
- Avoid ambiguity
- Natural pause between sections
- Same logical structure as text
- No filler language, no speculation, no invented claims
- Speak from PH1.WRITE `formatted_text` with clear pacing and pronunciation-safe delivery

## 5) Determinism and Safety

- Fixed provider timeouts
- Fixed max result count
- Deterministic merge order (lead first, assist next)
- Deterministic dedup key
- Deterministic ranking function

Fail-closed rules:

- Missing key -> `provider_unconfigured` + vault hint
- Upstream fail -> `tool_upstream_failed` + safe `fail_detail`
- Empty results -> explicit `no results`

Proxy handling (universal):

Selene supports three deterministic modes:

- `off`: direct internet connection
- `env`: uses `HTTP_PROXY` and `HTTPS_PROXY`; ignores `ALL_PROXY`
- `explicit`: requires `SELENE_HTTP_PROXY_URL` and `SELENE_HTTPS_PROXY_URL`

If proxy is misconfigured:

- return `fail_detail` including `proxy_mode` + safe host:port
- do not retry endlessly

## 6) Implementation Phases

### Run 1 — URL Fetch Minimal (Do Now)

Goal: Open a real URL and extract readable text so Selene can cite real page content (before chunking).

Run gates:

- Start gate: `git status --porcelain` must be empty before run work begins.
- End gate: commit run changes, `git push origin main`, and prove clean tree (`git status --porcelain` empty).

What Selene does:

- Input: single URL (user-provided or top-K selection)
- Fetch: HTTP GET via current proxy mode (`off` / `env` / `explicit`)
- Extract: HTML -> readable text (strip scripts/styles, normalize whitespace)
- Bound: enforce `max_total_extracted_chars` hard cap

Output (Evidence Packet):

- `sources[]` includes URL (title optional)
- `retrieved_at`
- `extracted_text_preview` (bounded)
- `page_hash` (optional for Run 1)

Not in Run 1:

- No chunking
- No `chunk_id`
- No PH1.D synthesis
- No conflict handling

Acceptance:

- `UrlFetchAndCite` returns non-empty extracted text for a real URL
- Failures return safe `fail_detail` (`connection` / `tls` / `dns` / `http status`)
- Output always includes `sources[]` and `retrieved_at`

### Run 1.1 — Proxy Universal Rules (Required)

Selene must support:

- Start gate: `git status --porcelain` must be empty before run work begins.
- End gate: commit run changes, `git push origin main`, and prove clean tree (`git status --porcelain` empty).

- `off`: direct internet
- `env`: `HTTP_PROXY` + `HTTPS_PROXY` only (`ALL_PROXY` ignored)
- `explicit`: `SELENE_HTTP_PROXY_URL` + `SELENE_HTTPS_PROXY_URL` required

When proxy is misconfigured:

- return `fail_detail` with `proxy_mode` + safe host:port
- never loop retries endlessly (cooldown)

### Run 2 — Chunk + Hash + Citation IDs

Goal: Convert extracted page text into deterministic, citeable units.

Run gates:

- Start gate: `git status --porcelain` must be empty before run work begins.
- End gate: commit run changes, `git push origin main`, and prove clean tree (`git status --porcelain` empty).

What Selene does:

- Deterministic chunk split
- `chunk_id = stable hash(chunk_text)`
- Store `content_chunks[]`:
- `chunk_id`
- excerpt (bounded)
- `source_url`
- Generate citations referencing `chunk_id(s)`

Acceptance:

- Deterministic chunk ordering
- Deterministic `chunk_id` generation
- Citation references can deterministically bind to `chunk_id`s once synthesis is enabled in Run 3

### Run 3 — PH1.D Grounded Synthesis (Mandatory)

Goal: Produce final user response from evidence only.

Run gates:

- Start gate: `git status --porcelain` must be empty before run work begins.
- End gate: commit run changes, `git push origin main`, and prove clean tree (`git status --porcelain` empty).

PH1.D inputs:

- user question
- Evidence Packet only

PH1.D outputs (structured):

- Direct answer (short)
- Bullet evidence (each bullet cites >=1 source or `chunk_id`)
- Citations list
- Uncertainty/conflict notice when needed
- One clarifying question only when strictly necessary

PH1.WRITE requirements:

- Text output: headings/bullets/citations clearly formatted
- Voice output: same structure, spoken clearly in requested language from the same PH1.WRITE `formatted_text`

Fail-closed:

- If Evidence Packet does not support claim -> refuse or state `Insufficient evidence found`

### Run 4 — Search -> Open Top K URLs -> Grounded Answer

Run gates:

- Start gate: `git status --porcelain` must be empty before run work begins.
- End gate: commit run changes, `git push origin main`, and prove clean tree (`git status --porcelain` empty).

- Brave search returns URL/snippet candidates
- Deterministically select top K URLs (e.g., 2)
- Fetch + extract + chunk each URL
- Build Evidence Packet v2 with page-level chunks
- PH1.D synthesizes from page chunks, not snippets alone

### Run 5 — Images + Video (Implement Now)

Run gates:

- Start gate: `git status --porcelain` must be empty before run work begins.
- End gate: commit run changes, `git push origin main`, and prove clean tree (`git status --porcelain` empty).

Add:

- `ImageSearchQuery`
- `VideoSearchQuery`

Provider:

- Brave Image Search
- Brave Video Search

Output:

- metadata + URLs only
- citations included
- no download/playback hosting

Acceptance:

- non-empty results + real URLs
- deterministic ordering
- safe `fail_detail` on upstream failure

### Run 6 — News Assist (GDELT) — Implement Now

News retrieval:

- Start gate: `git status --porcelain` must be empty before run work begins.
- End gate: commit run changes, `git push origin main`, and prove clean tree (`git status --porcelain` empty).

- Brave News (Lead)
- GDELT (Assist)
- GDELT is active assist in this run (not future-only)

Deterministic merge:

- dedup by `canonical_url`
- Brave order first
- then GDELT uniques

## 6.1) Run-by-Run Gap Closure Checklist (Global Standard)

This checklist is normative for build quality. A run is not complete unless its gap-closure checks are satisfied.

### Run 1 — Contract Lock
- Start gate: `git status --porcelain` must be empty before run work begins.
- End gate: commit run changes, `git push origin main`, and prove clean tree (`git status --porcelain` empty).
- Define explicit schema versioning policy for all core packet contracts.
- Define backward-compatibility rules (reader/writer behavior across versions).
- Define one canonical reason-code registry source for this plan.
- Add contract conformance examples for expected/invalid payloads.
- Assign field ownership per contract surface (engine/module owner map).

### Run 2 — Proxy Universal Layer
- Start gate: `git status --porcelain` must be empty before run work begins.
- End gate: commit run changes, `git push origin main`, and prove clean tree (`git status --porcelain` empty).
- Pin numeric retry and cooldown limits (exact values).
- Pin TLS failure classification policy (`tls`, `dns`, `connection`, `timeout`, `http_non_200`).
- Enforce proxy-auth redaction guarantees in all diagnostics.
- Define startup self-check severity levels and expected operator action.
- Define deterministic outage behavior (no hidden retries, fail-closed response path).

### Run 3 — URL Fetch Minimal
- Start gate: `git status --porcelain` must be empty before run work begins.
- End gate: commit run changes, `git push origin main`, and prove clean tree (`git status --porcelain` empty).
- Define content-type allowlist (HTML/text and explicit exclusions).
- Define max redirect policy (count + terminal behavior).
- Define charset/encoding normalization rules.
- Define robots/compliance guard behavior for fetch eligibility.
- Define unsafe HTML/script/style handling boundaries before extraction.

### Run 4 — Chunk/Hash/Citation IDs
- Start gate: `git status --porcelain` must be empty before run work begins.
- End gate: commit run changes, `git push origin main`, and prove clean tree (`git status --porcelain` empty).
- Define canonical normalization algorithm prior to chunking/hashing.
- Pin hash algorithm and hash-version field.
- Define hash collision handling policy.
- Define deterministic truncation strategy for excerpts and bounds.
- Add replay fixtures proving stable chunk ordering and IDs for same input.

### Run 5 — PH1.D Grounded Synthesis
- Start gate: `git status --porcelain` must be empty before run work begins.
- End gate: commit run changes, `git push origin main`, and prove clean tree (`git status --porcelain` empty).
- Define claim-to-citation validator requirements.
- Define conflict-resolution scoring/selection policy.
- Define insufficient-evidence threshold and refusal trigger.
- Pin deterministic synthesis template version.
- Prove PH1.D cannot emit unsupported claims under contract validation.

### Run 6 — PH1.WRITE Structured Output
- Start gate: `git status --porcelain` must be empty before run work begins.
- End gate: commit run changes, `git push origin main`, and prove clean tree (`git status --porcelain` empty).
- Define localization/i18n output contract (text + voice).
- Define deterministic voice pause/prosody mapping rules.
- Define accessibility/readability constraints for formatted text.
- Define output-length budgets and deterministic truncation behavior.
- Verify parity of text and voice logical structure under same evidence.

### Run 7 — Web Provider Ladder
- Start gate: `git status --porcelain` must be empty before run work begins.
- End gate: commit run changes, `git push origin main`, and prove clean tree (`git status --porcelain` empty).
- Define exact fallback trigger thresholds from Brave to OpenAI.
- Define per-provider timeout matrix and enforcement points.
- Define provider-health state model used for routing decisions.
- Define anti-hallucination checks for fallback-only evidence conditions.
- Prove provider-run trace rows are persisted deterministically.

### Run 8 — News Provider Ladder
- Start gate: `git status --porcelain` must be empty before run work begins.
- End gate: commit run changes, `git push origin main`, and prove clean tree (`git status --porcelain` empty).
- Define recency window policy for news ranking.
- Define publish-date normalization and missing-date handling.
- Define canonical URL reconciliation for redirects/paywalls/trackers.
- Define contradiction-group assignment policy for conflicting reports.
- Prove deterministic Brave-first then GDELT-unique merge ordering.

### Run 9 — Search → Top-K Open → Grounded Answer
- Start gate: `git status --porcelain` must be empty before run work begins.
- End gate: commit run changes, `git push origin main`, and prove clean tree (`git status --porcelain` empty).
- Define exact Top-K selection formula and tie-breakers.
- Define per-domain/page-open caps and enforcement behavior.
- Define dead-link replacement/fallback policy deterministically.
- Define deterministic stop conditions for URL opening.
- Prove synthesis uses page-level chunk evidence, not snippet-only drift.

### Run 10 — Image/Video Retrieval
- Start gate: `git status --porcelain` must be empty before run work begins.
- End gate: commit run changes, `git push origin main`, and prove clean tree (`git status --porcelain` empty).
- Define media metadata schema lock (required/optional fields).
- Define safe-search policy and enforcement behavior.
- Define duplicate-media canonicalization strategy.
- Define unsupported-media fail-closed behavior.
- Prove deterministic ordering and citation attachment for media results.

### Run 11 — Learning Layer
- Start gate: `git status --porcelain` must be empty before run work begins.
- End gate: commit run changes, `git push origin main`, and prove clean tree (`git status --porcelain` empty).
- Define failure-signature schema and versioning policy.
- Define proposal artifact format for builder-governed promotion.
- Define approval SLA/gates before ACTIVE change.
- Define rollback criteria and rollback proof requirements.
- Prove authority-path invariants are unchanged by learning actions.

### Run 12 — Test + Acceptance Closure
- Start gate: `git status --porcelain` must be empty before run work begins.
- End gate: commit run changes, `git push origin main`, and prove clean tree (`git status --porcelain` empty).
- Map mandatory test names to CI scripts and gate conditions.
- Add reliability/performance acceptance thresholds.
- Add full replay-determinism suite for key flows.
- Define release-readiness checklist with explicit pass/fail thresholds.
- Require final evidence pack (commands, outputs, ledger refs) before close.

## 7) Budgets and Deterministic Limits

- `max_results_from_search` (e.g., 5)
- `max_urls_opened_per_query` K (e.g., 2)
- `max_total_extracted_chars`
- `max_chunks_total`
- per-provider timeout
- total timeout per request
- deterministic URL canonicalization

Hard limits:

- never open unlimited pages
- never extract unlimited content

## 8) Brave -> OpenAI Fallback Behavior (Required)

Web + News:

- Primary: Brave
- If Brave fails (`connection` / `tls` / `dns` / `http_non_200` / `timeout` / `empty_results`), OpenAI `web_search` runs as fallback.
- Evidence Packet must record `provider_runs: brave failed`.
- Evidence Packet must record `provider_runs: openai used`.
- PH1.D synthesizes only from returned Evidence Packet.
- OpenAI must return sources that are cited explicitly.

OpenAI fallback must not:

- guess
- invent citations
- bypass evidence rules

## 9) Learning — Two-Layer Model

### A) Instant Learning (Session-Level)

- Stop retry storms
- Apply cooldown for repeated failure signatures
- Adjust fallback ordering temporarily
- Apply immediate corrections within session

### B) Builder-Governed Learning

- Provider routing changes
- Timeout tuning changes
- Prompt template updates
- NLP rule changes

Only Builder can promote changes to ACTIVE.

Learning cannot:

- Modify simulations
- Bypass access
- Change authority paths

## 10) Required Tests

- UrlFetchAndCite extracts readable text
- Deterministic `chunk_id` stability
- PH1.D refuses unsupported claims
- Brave fail -> OpenAI fallback success
- Brave + GDELT deterministic merge
- ImageSearch + VideoSearch routing
- Proxy mode `off` / `env` / `explicit` behavior
- `fail_detail` correctness
- Failure signature cooldown logic

## 11) Acceptance Criteria

### Web
- Non-empty sources
- Answer includes citations
- No invented claims

### News
- Recent articles
- Publish dates when available
- Conflict handling

### Image/Video
- Real URLs returned
- Metadata preserved

### URL Fetch
- Extracted content present
- Chunk hashes stable
- Citations mapped

### PH1.D
- Grounded answer only
- Refuses unsupported claims

## 12) PH1.D Response Style Rules (Reference Only)

Canonical response-style requirements are defined in Section 4.1 (`PH1.WRITE Structured Output Rules`).  
This section is intentionally non-normative to avoid dual truth.

## 13) Failure Learning Layer (Simple Overview)

Purpose: Selene records mistakes, analyzes patterns, proposes improvements without auto-changing execution.

Governance boundaries are canonical in Section 9 (`Learning — Two-Layer Model`).

Failure examples:

- proxy misconfig (`provider=brave error=connection proxy_mode=off`)
- user correction (`intent_misclassify` or `synthesis_drift`)
- repeated empty results

Each failure is written to append-only Failure Ledger.

Offline analysis:

- detect repeated patterns
- generate proposal artifacts (example: recommend explicit proxy mode)
- Builder approval required before ACTIVE changes

Repeat-failure prevention:

- detect repeated failure signature within window
- stop retry loops
- return clear config hint
- avoid wasted external calls

In one sentence: Selene logs mistakes, finds patterns, suggests improvements, and only changes behavior after governed approval.

## 14) Run-by-Run Functionality Upgrade Addendum (Global Standard)

This section is additive and does not replace existing run requirements.

### Run 1 — Contract Lock
- Add contract hash/version manifest.
- Add backward-compat matrix with allowed and blocked changes.
- Add canonical reason-code map file reference.
- Add golden valid/invalid packet fixtures.
- Add CI contract-lint gate.

### Run 2 — Proxy Universal Layer
- Add deterministic proxy error taxonomy coverage tests.
- Add startup self-check severity levels (`warn` vs `critical`).
- Add safe diagnostic rate-limiting.
- Add proxy auth-redaction tests.
- Add deterministic cooldown/backoff table.

### Run 3 — URL Fetch Minimal
- Add MIME allowlist plus reject list.
- Add redirect depth plus loop protection rules.
- Add charset normalization policy.
- Add decompression and size caps for streamed reads.
- Add extraction-quality threshold with fail-closed reason.

### Run 4 — Chunk/Hash/Citation IDs
- Add canonical text normalization spec before hashing.
- Add pinned hash algorithm plus `hash_version`.
- Add collision-handling policy.
- Add deterministic truncation rules.
- Add replay fixtures proving same input equals same chunk IDs.

### Run 5 — PH1.D Grounded Synthesis
- Add claim-to-citation coverage validator (`100% cited claims`).
- Add contradiction grouping logic with confidence labels.
- Add insufficient-evidence refusal thresholds.
- Add deterministic template version pin.
- Add unsupported-claim hard-fail test set.

### Run 6 — PH1.WRITE Structured Output
- Add formal output schema for text and voice render.
- Add accessibility/readability constraints.
- Add deterministic citation placement rules.
- Add localization behavior contract.
- Add style conformance tests (no filler/speculation).

### Run 7 — Web Provider Ladder
- Add exact Brave-to-OpenAI fallback trigger thresholds.
- Add per-provider timeout and budget matrix.
- Add provider health-state transitions.
- Add fallback-only anti-hallucination checks.
- Add deterministic provider-run audit persistence checks.

### Run 8 — News Provider Ladder
- Add recency window policy by topic class.
- Add publish-date normalization/timezone policy.
- Add source-diversity minimum rule.
- Add contradiction cluster assignment policy.
- Add deterministic Brave-first plus GDELT-unique merge proofs.

### Run 9 — Search to Top-K Open to Grounded Answer
- Add fixed Top-K scoring and tie-break formula.
- Add per-domain open caps.
- Add dead-link replacement rules.
- Add deterministic stop conditions when open budget is exhausted.
- Add snippet-only fallback behavior when URL open fails.

### Run 10 — Image/Video Retrieval
- Add metadata schema lock (required/optional fields).
- Add safe-search policy and rejection reasons.
- Add duplicate canonicalization for media URLs.
- Add deterministic ordering rules by rank/provider.
- Add citation-required checks for every media item.

### Run 11 — Learning Layer
- Add failure-signature schema plus TTL/versioning.
- Add proposal-priority scoring policy.
- Add bounded session-level adaptation limits.
- Add builder approval SLA and proof requirements.
- Add rollback-drill requirements and rollback verification tests.

### Run 12 — Test + Acceptance Closure
- Add CI matrix mapping run to tests to scripts.
- Add reliability/performance thresholds.
- Add replay determinism suite as mandatory.
- Add release readiness checklist with hard pass/fail gates.
- Add final evidence pack format (commands, outputs, ledger refs).

## 15) Cross-Run Excellence Controls

This section is additive and applies across all runs.

- Add SLO targets (`availability`, `latency`, `citation coverage`, `fallback rate`, `refusal correctness`).
- Add PH1.J audit completeness checklist (every turn persists query hash, evidence hash, response hash, provider runs, reason code).
- Add privacy/compliance controls (PII redaction policy in logs and `fail_detail`).
- Add cost controls (per-turn provider budget, fallback cap, open-URL cap).
- Add operational controls (runbook links, alert thresholds, incident response ownership).
- Add governance controls (every run must include clean-start proof, clean-end proof, ledger entry, and push proof).

## 16) System Wiring Plan

Scope note:
- This section is a conceptual wiring summary.
- Authoritative final wiring/ownership contracts for the finished build are defined in Section `40`.

### 16.1 Canonical Wiring Law

1. `PH1.OS` is orchestration authority.
2. `PH1.X` is conversation-move authority.
3. `PH1.E` is read-only tool authority.
4. `PH1.D` is synthesis-only (non-authoritative).
5. `PH1.WRITE` is presentation-only.
6. `PH1.J` is audit truth.
7. Engines never call engines directly; Selene OS orchestrates all handoffs.

### 16.2 End-to-End Turn Wiring (Single Turn)

1. Input capture: `PH1.W -> PH1.K -> PH1.C`.
2. Transcript repair: `PH1.SRL` (plus `PH1.LANG` optional).
3. Context build: `PH1.CONTEXT` (plus `PH1.KNOW` and `PH1.CACHE` optional hints).
4. Intent extraction: `PH1.NLP -> PH1.PRUNE` / `PH1.DIAG` checks.
5. Decision: `PH1.X` chooses `Clarify | Refuse | Tool`.
6. Search assist path: `PH1.SEARCH` rewrites and ranks query intent.
7. Retrieval path: `PH1.E` executes provider ladder plus URL fetch/chunking.
8. Evidence lock: `EvidencePacket` returned; `PH1.D` can use only this.
9. Synthesis path: `PH1.D` builds grounded answer plus citations.
10. Output path: `PH1.WRITE` formats text/voice structure, `PH1.TTS` speaks if needed.
11. Audit path: `PH1.J` appends query/evidence/response/proof hashes.
12. Learning path: `PH1.FEEDBACK -> PH1.LEARN -> PH1.PAE` (proposal/hints only; no runtime authority mutation).

### 16.3 Run-by-Run Wiring (Build Sequence)

| Run | Primary Engines | Wiring Output |
|---|---|---|
| Run 1 URL Fetch Minimal | `PH1.X -> PH1.SEARCH -> PH1.E -> PH1.J/PH1.F` | Real URL fetch Evidence Packet (no synthesis) |
| Run 1.1 Proxy Universal | `PH1.E + adapter startup + PH1.J` | Deterministic proxy modes + safe diagnostics |
| Run 2 Chunk/Hash | `PH1.E -> PH1.F/PH1.J` | Stable chunk IDs + citation references |
| Run 3 Grounded Synthesis | `PH1.X -> PH1.D -> PH1.WRITE -> PH1.J` | Evidence-only answer with strict citation discipline |
| Run 4 Top-K Open | `PH1.SEARCH -> PH1.E -> PH1.D -> PH1.WRITE -> PH1.J` | Search to page-level grounding flow |
| Run 5 Image/Video | `PH1.X/PH1.SEARCH -> PH1.E -> PH1.D -> PH1.WRITE -> PH1.J` | Metadata+URL media results with citations |
| Run 6 News Assist GDELT | `PH1.SEARCH -> PH1.E -> PH1.D -> PH1.WRITE -> PH1.J` | Brave-first + GDELT deterministic merge |
| Gap Runs 1-12 | Core path plus `PH1.FEEDBACK/PH1.LEARN/PH1.PAE/PH1.GOV` | Hardening, replay, governance, CI closure |

## 17) Ownership Matrix

### 17.1 Engine Ownership Matrix (Web/News System)

| Engine | Ownership | Function | Must Not Do |
|---|---|---|---|
| `PH1.OS` | Authoritative orchestration | Pipeline ordering + legality | Execute tools directly |
| `PH1.X` | Authoritative conversation move | Decide next action and lane | Synthesize facts |
| `PH1.SEARCH` | Non-authoritative planning assist | Query rewrite, source interpretation hints | Call providers directly |
| `PH1.E` | Authoritative read-only tools | Web/news/image/video/url fetch, evidence build | Decide conversation final text |
| `PH1.D` | Non-authoritative understanding | Grounded synthesis from Evidence Packet only | Use outside knowledge |
| `PH1.WRITE` | Non-authoritative output | Structured formatting only | Change semantic claims |
| `PH1.TTS` | Authoritative playback | Voice rendering/cancel safety | Alter answer content |
| `PH1.J` | Authoritative audit | Append-only proofs/events | Business decision logic |
| `PH1.F` | Authoritative persistence | Store evidence/projections/idempotency | Runtime policy decisions |
| `PH1.CACHE` | Non-authoritative assist | Cache hints/snapshots | Override evidence truth |
| `PH1.NLP` | Non-authoritative assist | Intent/fields draft | Final move authority |
| `PH1.PRUNE` | Non-authoritative assist | Single best missing-field target | Ask multi-question loops |
| `PH1.DIAG` | Non-authoritative assist | Consistency/safety pre-check | Override `PH1.X` authority |
| `PH1.C` | Authoritative transcript gate | STT quality gate | Intent dispatch |
| `PH1.SRL` | Non-authoritative assist | Deterministic semantic repair | Intent drift |
| `PH1.CONTEXT` | Non-authoritative assist | Bounded context assembly | Execute side effects |
| `PH1.KNOW` | Non-authoritative assist | Tenant vocab/pron hints | Decide final claims |
| `PH1.FEEDBACK` | Non-authoritative assist | Capture correction signals | Live behavior mutation |
| `PH1.LEARN` | Non-authoritative assist | Aggregate adaptation artifacts | Activate changes |
| `PH1.PAE` | Non-authoritative assist | Provider scoring hints | Change lead/assist at runtime |
| `PH1.GOV` / Builder | Authoritative governance | Promote/demote artifacts/policies | Bypass audit/access |
| `PH1.ACCESS` | Authoritative access gate | Allow/deny/escalate decisions | Execute retrieval |
| `PH1.POLICY` | Authoritative policy decision | Global policy snapshot enforcement | Tool execution |
| `PH1.QUOTA` / `PH1.COST` | Authoritative/non-authoritative mixed | Budget pacing + caps + degrade hints | Invent evidence |

### 17.2 Authority and Boundary Notes

- `PH1.D` and `PH1.WRITE` are strictly non-authoritative and cannot alter control decisions.
- `PH1.E` remains the only read-only tool execution authority in this plan.
- `PH1.X` remains the one-turn conversation move authority.
- `PH1.J` is the canonical append-only audit proof lane.

### 17.3 Wire-Ready Addenda (Must Be Added for Full Build Readiness)

Status:
- These addenda are now captured by Sections `38` and `40` and are treated as implementation requirements, not optional future notes.

Fulfillment mapping:
1. Canonical packet registry requirement -> Section `38.1`
2. Canonical ownership appendix requirement -> Section `40.6`
3. Canonical handoff map requirement -> Section `40.4`
4. Canonical failure ownership map requirement -> Section `40.7`
5. Canonical idempotency key table requirement -> Section `38.3`
6. CI gate map requirement -> Section `38.5` plus Section `39.7`

## 18) Performance Budget Contract

- Per-turn latency SLO budget split by stage (`X`, `SEARCH`, `E`, `D`, `WRITE`, `TTS`).
- Hard timeout envelope and deterministic degradation order.
- Max concurrent outbound fetches and queue/backpressure policy.

### 18.1 Search Importance Tier Policy (Deterministic)

- Default tier is `medium`.
- User language can request tier changes:
- quick-check intent (`"quick check"`, similar) -> `low`
- deep/accuracy intent (`"deep search"`, `"need accurate info"`, similar) -> `high`
- User profile/personality may set default bias only (for example, prefer concise vs thorough), but does not override explicit user instruction.
- Explicit user request always overrides profile bias for that turn.
- Tier selection must be persisted in audit/provenance (`PH1.J`) for replay and traceability.
- Each tier must use bounded deterministic caps:
- timeout envelope
- max provider fan-out
- max URLs opened
- max chunks synthesized
- Personality and profile influence depth preference only; they must not alter grounding, citation requirements, access decisions, or safety rules.

## 19) Parallel Retrieval Strategy

- Deterministic parallel fan-out for provider calls and URL fetches.
- Bounded concurrency caps per domain/provider.
- Stable merge ordering after parallel completion.

## 20) Two-Tier Cache Strategy

- L1 in-turn ephemeral cache plus L2 short TTL cache for provider results.
- Canonical cache key schema and invalidation rules.
- Cache safety rule: cache may improve latency, never alter semantic truth.

## 21) Query Planning Optimizer

- Deterministic query rewrite templates by intent class.
- Domain-aware rewrite hints (news vs web vs image/video).
- Rewrite confidence and fallback-to-original rule.

## 22) URL Quality Gate

- Pre-open URL scoring (trust, freshness, canonicalization, duplication).
- Blocklist/allowlist and low-value URL rejection.
- Deterministic Top-K promotion and replacement.

## 23) Evidence Compression and Dedup

- Snippet/chunk dedup before synthesis using canonical text fingerprint.
- “Same claim cluster” compaction to reduce synthesis load.
- Keep citation traceability after compaction.

## 24) Streaming Response Mode

- Optional partial response contract (header -> evidence bullets -> citations).
- Deterministic partial ordering and completion markers.
- Abort/cancel handling with clean audit closure.

## 25) Domain Function Packs

- Pack-specific logic for finance, legal, health, travel, product comparisons.
- Domain-specific source ranking and refusal thresholds.
- Pack activation via governance only (no runtime drift).

## 26) Multilingual Retrieval/Synthesis

- Query translation strategy with source-language preservation.
- Cross-language citation mapping and displayed-language policy.
- Deterministic language fallback if translation confidence is low.

## 27) Advanced Conflict Resolver

- Claim graph across sources with contradiction severity levels.
- Source-weight plus recency-weight policy.
- Mandatory conflict-summary format in final output.

## 28) Personalization Without Authority Drift

- User preference hints (brevity/detail/style) only at `WRITE` layer.
- Hard rule: personalization cannot alter ranking, evidence, or citations.
- Preference fallback when conflicts with safety/grounding policy.

## 29) Real-Time Health Routing

- Provider health score integration into routing decisions (deterministic).
- Cooldown expiry and recovery rules.
- Health-state audit rows in `PH1.J`.

## 30) Observability and Debug Packet

- One standard debug packet schema for failed turns.
- Required safe fields (`provider`, `error_kind`, `status`, `proxy_mode`, `trace_id`).
- No-secret redaction rules and retention policy.

## 31) Cost-Aware Planning

- Per-turn spend cap with deterministic fallback tiers.
- Model/provider selection policy by budget plus urgency.
- Cost anomaly detection plus governance report.

## 32) Benchmark and Replay Harness

- Golden benchmark suite for speed/quality.
- Deterministic replay corpus (same input => same output envelope).
- Regression gates for latency, citation coverage, and refusal correctness.

## 33) ChatGPT Capability Parity Enhancements (Additive)

This section is additive and does not replace existing requirements.

1. Multi-query decomposition
- Split one request into 2-4 deterministic sub-queries and merge results.

2. Source diversification policy
- Require cross-domain source mix before final synthesis on high-importance queries.

3. Query reformulation ladder
- Retry with structured rewrites when first search underperforms.

4. Answer-first with expandable evidence UX
- Short direct answer with optional deeper evidence expansion layers.

5. Better multilingual retrieval loop
- Dual-language query strategy with language-aware ranking.

6. Retrieval quality reranker
- Deterministic rerank step using relevance plus trust plus freshness weights.

7. Ambiguity-first clarification policy
- One best clarifying question selected by highest uncertainty reduction.

8. Domain-specific reasoning templates
- Finance/legal/health/news template packs with stricter evidence rules.

9. Continuous eval suite
- Daily benchmark turns for accuracy, citation coverage, latency, and refusal correctness.

10. Adaptive presentation modes
- Same truth, multiple formats (`brief` / `standard` / `deep`) selected by importance tier.

11. Robust fallback stitching and contradiction summary
- Merge partial provider outputs safely when one source fails.
- Explicitly show: what agrees, what conflicts, and what remains unknown.

## 34) Additional Capability Gap Closers (Additive)

This section is additive and does not replace existing requirements.

1. Memory-grounded follow-up continuity
- Preserve grounded follow-up continuity across long sessions.

2. User-correction learning loop
- Apply user correction feedback so the next turn can deterministically fix prior mistakes.

3. Why-this-answer transparency mode
- Add a trace mode that explains why each claim was selected.

4. Per-claim confidence calibration
- Emit confidence labels per claim, not just per response.

5. Multi-hop research planner
- Add explicit subgoal tree planning for complex research queries.

6. Source freshness watchdog
- Re-check stale citations and refresh evidence when freshness thresholds are exceeded.

7. Structured table/chart rendering
- Provide high-quality table/chart answer formats grounded to evidence.

8. Prompt-injection defense policy
- Add strict prompt-injection defenses for fetched page content.

9. Cross-source consensus scoring
- Score agreement strength across sources before final synthesis.

10. Unknown-first behavior
- Prefer explicit unknown/insufficient responses when confidence is below threshold.

11. Human handoff packet
- Build a deterministic handoff packet when answer risk is high.

12. Conversation objective tracking
- Track user objective across turns so long tasks do not drift.

13. Mid-turn evidence revision
- Allow deterministic answer revision when stronger evidence arrives during a turn.

14. Long-document compression upgrades
- Add section-aware compression and evidence mapping for long documents.

15. Personal knowledge boundary control
- Define strict controls for what memory context can and cannot influence.

## 35) Enterprise Research Expansion (Requested Scope, Additive)

This section directly incorporates enterprise-grade requirements and is additive to Sections `1-34`.

### 35.1 Structured Data Research Mode (Not Pages Only)

Required source classes:
- company registry datasets
- financial filings and investor reports
- government databases and regulatory portals
- product specification documents
- pricing tables
- patents
- academic papers

Required capabilities:
- structured extraction mode (key-value, table, entity, metric extraction)
- table parsing mode with schema validation
- PDF extraction mode with section and table anchors
- filing-aware parsing mode (`10-K/10-Q`-style field packs where applicable)

Deterministic rule:
- if source is structured, extraction must produce typed rows before synthesis (not prose-only snippets).

### 35.2 Competitive Intelligence Mode

Required outputs:
- competitor comparison table
- feature differences
- pricing comparison
- market-position summary
- SWOT-style evidence-backed summary
- customer sentiment summary (source-scoped)

Deterministic requirements:
- multi-source synthesis with deduplicated feature entities
- canonical comparison schema (`entity`, `attribute`, `value`, `source_ref`, `as_of_date`)
- explicit unknown fields when evidence is missing

### 35.3 Real-Time Data Research Mode

Scope examples:
- stocks, crypto, weather, flights, commodities

Architecture rule:
- API-first retrieval for real-time domains; web-page search is assist/fallback only.

Deterministic controls:
- strict freshness TTL per domain
- retrieval timestamp and freshness score mandatory in Evidence Packet
- fail-closed if data freshness policy cannot be met

### 35.4 Regulatory / Compliance Research Mode

Required behavior:
- jurisdiction-aware retrieval and filtering
- confidence flagging (`HIGH|MODERATE|LOW`) per compliance claim
- trust-first source policy (official/regulatory sources preferred)

Regulatory trust policy:
- require explicit trust tier on every compliance citation
- refuse unsupported compliance claims with `insufficient_regulatory_evidence`

### 35.5 Source Trust and Ranking Model (Canonical)

Source ranking must include:
- domain reputation score
- official-source detection (`.gov`, regulator domains, official filings registries, recognized academic domains)
- spam/clickbait filtering
- recency and corroboration weighting

Deterministic ranking vector:
- `final_rank = w1*relevance + w2*trust + w3*freshness + w4*corroboration - w5*spam_risk`
- integer weights are policy-snapshot controlled and versioned.

### 35.6 Deep Research (Multi-hop) Mode

Required behavior:
- explicit subgoal chain (`hop_1..hop_n`)
- each hop produces evidence refs consumed by next hop
- terminal answer requires completed hop proof chain

Hard rule:
- no hidden hop; every hop must be auditable in `PH1.J`.

### 35.7 Temporal Comparison Mode

Required behavior:
- date-filtered retrieval windows
- timeline extraction
- change detection between snapshots/reports

Outputs:
- `what_changed`, `what_unchanged`, `first_seen`, `latest_seen`, citation refs

### 35.8 Risk Scoring Mode

Risk inputs (domain-dependent):
- financial stress indicators
- lawsuits/legal events
- regulatory events
- negative news clustering
- operational reliability signals

Deterministic outputs:
- composite risk score with factor breakdown
- confidence and evidence coverage indicators
- explicit non-advice guardrails where required

### 35.9 Internal + External Merge Mode

Required behavior:
- merge internal memory/context (`PH1.M` / `PH1.CONTEXT`) with external evidence
- produce explicit delta format:
  - `previous_internal_view`
  - `new_external_findings`
  - `changes_since_last_time`

Hard boundary:
- internal memory can guide retrieval focus, but cannot override external evidence truth.

### 35.10 Enterprise Report Mode

Report templates (deterministic):
- executive summary
- key findings
- risk summary
- comparison tables
- sources and appendix

Output modalities:
- text report (PH1.WRITE)
- voice summary (PH1.TTS from same canonical formatted summary)
- optional graph/table/image/video embeds as referenced artifacts (metadata + source refs)

### 35.11 Search Decision + Conversational Uncertainty Handling

Required behavior:
- deterministic router decides `search` vs `no-search` vs `clarify`
- dynamic query rewrite ladder when first retrieval underperforms
- uncertainty handling must be conversational but evidence-bounded

Hard rule:
- no confident claim without citation support, even after rewrite or fallback.

## 36) Run Integration Plan for Enterprise Expansion (Additive)

This run map ensures every requested capability is scheduled. Existing runs remain valid; these additions extend them.

### 36.1 Fit Into Existing Runs

| Requested capability | Existing run(s) extended |
|---|---|
| Structured extraction/table/PDF basics | Run 1, Run 2, Run 4 |
| Competitive comparison outputs | Run 3, Run 4, Run 6 |
| Real-time data freshness controls | Run 1.1, Run 7, Run 8 |
| Regulatory trust filtering | Run 6, Run 8 |
| Trust/ranking model | Run 7, Run 8, Run 9 |
| Multi-hop research | Run 9, Run 11 |
| Temporal comparison | Run 4, Run 9 |
| Risk scoring | Run 3, Run 11 |
| Internal+external merge | Run 3, Run 11 |
| Enterprise report template mode | Run 3, Run 6, Run 12 |

### 36.2 New Additive Runs (Required)

#### Run 13 — Structured Data Connectors and Parsing
- Add structured source adapters (registries, filings, official datasets).
- Implement table + PDF + filing-aware extraction contracts.
- Add tests for deterministic extraction and schema-stable output.

#### Run 14 — Competitive Intelligence Pipeline
- Implement canonical comparison schema and dedup entity merger.
- Add competitor/pricing/feature comparison outputs with citations.
- Add tests for deterministic comparison ranking and unknown-field behavior.

#### Run 15 — Real-Time API Lane + Freshness Policy
- Add API-first providers for real-time domains.
- Add freshness TTL policy and fail-closed behavior.
- Add tests for stale-data refusal and timestamp correctness.

#### Run 16 — Regulatory Trust Tier and Jurisdiction Routing
- Add jurisdiction resolver and regulatory trust policy.
- Enforce official-source preference and compliance confidence flags.
- Add tests for jurisdiction mismatch and low-trust refusal paths.

#### Run 17 — Trust/Spam/Official Source Model
- Implement trust score, official-source detection, spam/clickbait penalties.
- Wire deterministic rank vector into retrieval ordering.
- Add tests for trust-priority ordering and spam suppression.

#### Run 18 — Multi-Hop Deep Research Planner
- Add deterministic hop planner and hop-proof chain persistence.
- Add stop conditions and max-hop budget.
- Add tests for reproducible multi-hop outputs and no-hidden-hop audit.

#### Run 19 — Temporal Comparison and Change Detection
- Add date-window retrieval controls and timeline extraction.
- Add deterministic change-diff output with citations.
- Add tests for `as_of` comparisons and multi-period report diffs.

#### Run 20 — Risk Scoring Mode
- Add deterministic risk factor model and breakdown outputs.
- Add clustering for negative event concentration.
- Add tests for score reproducibility and insufficient-evidence refusal.

#### Run 21 — Internal+External Merge
- Add merge packet combining memory context and external evidence.
- Enforce boundary: memory guides retrieval only, never overrides evidence truth.
- Add tests for delta summaries and memory-isolation safety.

#### Run 22 — Enterprise Report Templates and Delivery
- Add long-form report templates (exec summary/findings/risk/appendix).
- Add deterministic graph/table artifact references and citation mapping.
- Add tests for report completeness, citation coverage, and voice summary parity.

### 36.3 Completion Rule for Runs 13-22

Every run must:
- start with clean tree (`git status --porcelain` empty)
- end with commit + `git push origin main`
- end with clean tree proof (`git status --porcelain` empty)
- append ledger proof line with exact commands/tests executed

## 37) Run Completion Hardening Matrix (Additive, Mandatory)

This section closes run-level completeness gaps and is additive to Sections `6`, `6.1`, `14`, and `36`.

### 37.1 Canonical Run ID Map (Single Source for Execution Order)

Canonical execution order:
- `Run 1`
- `Run 1.1`
- `Run 2`
- `Run 3`
- `Run 4`
- `Run 5`
- `Run 6`
- `Run 7`
- `Run 8`
- `Run 9`
- `Run 10`
- `Run 11`
- `Run 12`
- `Run 13`
- `Run 14`
- `Run 15`
- `Run 16`
- `Run 17`
- `Run 18`
- `Run 19`
- `Run 20`
- `Run 21`
- `Run 22`

Alias resolution rule:
- Section `6` defines Runs `1..6`.
- Section `6.1` and Section `14` provide hardening details and are interpreted as extensions of the same canonical runs.
- Section `36` defines additive enterprise Runs `13..22`.
- If any run naming conflict appears in future edits, this section is authoritative for sequencing.

### 37.2 Run 1 Completion Upgrades

- Make `page_hash` mandatory (not optional) for every successful URL fetch output row.
- Define exact MIME allowlist and refusal reason for unsupported content types.
- Add deterministic redirect-loop detection with explicit fail-closed reason code.

### 37.3 Run 1.1 Completion Upgrades

- Define strict proxy precedence: `explicit` > `env` > `off`.
- Define unsupported proxy scheme behavior with fail-closed diagnostics.
- Add startup outbound probe PASS/FAIL thresholds and severity mapping.

### 37.4 Run 2 Completion Upgrades

- Lock canonical chunking algorithm (split logic + normalization pre-step).
- Add mandatory `hash_version` field beside each `chunk_id`.
- Add hash-collision handling rule with deterministic secondary key fallback.

### 37.5 Run 3 Completion Upgrades

- Pin `SynthesisPacket` schema version for PH1.D outputs.
- Add claim-to-citation validator gate (`100%` of factual claims must cite source/chunk).
- Add deterministic uncertainty threshold policy for `Low/Moderate/High` confidence labels.

### 37.6 Run 4 Completion Upgrades

- Lock Top-K selection formula with exact weighted fields and tie-breaks.
- Add domain diversity cap so one domain cannot dominate final evidence set.
- Add deterministic dead-link substitution policy when selected URLs fail to open.

### 37.7 Run 5 Completion Upgrades

- Lock image/video metadata schema (required vs optional fields).
- Add safe-search policy with tenant/profile policy snapshot control.
- Add canonical media URL normalization and duplicate collapse rules.

### 37.8 Run 6 Completion Upgrades

- Lock Brave+GDELT merge formula and precedence fields.
- Add publish-date normalization contract and timezone handling.
- Add contradiction clustering policy for conflicting news claims.

### 37.9 Run 7 Completion Upgrades

- Lock numeric fallback trigger thresholds for Brave -> OpenAI transition.
- Add per-turn provider budget guard and exhaustion behavior.
- Add fallback-only anti-hallucination test requirements.

### 37.10 Run 8 Completion Upgrades

- Add recency window policy by topic class (business/politics/regulatory/safety).
- Add minimum source-diversity requirement for news synthesis.
- Add paywall/redirect canonicalization handling and citation policy.

### 37.11 Run 9 Completion Upgrades

- Lock URL-open stop conditions and budget cutoff behavior by importance tier.
- Add per-tier open limits (`low`, `medium`, `high`) with deterministic caps.
- Add snippet-only fallback contract when all URL opens fail.

### 37.12 Run 10 Completion Upgrades

- Add media licensing/rights metadata requirement where available.
- Add explicit fail-closed reason codes for unsupported media/provider payloads.
- Add required citation linkage checks for every returned media item.

### 37.13 Run 11 Completion Upgrades

- Add feedback-poisoning resistance controls (replay abuse, malicious correction bursts).
- Add proposal-priority scoring formula for builder queue ordering.
- Add rollback drill cadence and verification evidence requirements.

### 37.14 Run 12 Completion Upgrades

- Add one release gate matrix (`test -> script -> threshold -> owner`).
- Add final evidence pack template with mandatory fields.
- Add mandatory gate: no release if any required evidence artifact is missing.

### 37.15 Run 13 Completion Upgrades

- Add structured schema registry for extracted datasets/tables/filings.
- Add parser-version pinning for structured extractors.
- Add table extraction conformance tests (typed columns + deterministic row ordering).

### 37.16 Run 14 Completion Upgrades

- Add canonical comparison output schema for competitive intelligence.
- Add deterministic unit/currency normalization rules.
- Add explicit unknown-field handling where evidence is absent.

### 37.17 Run 15 Completion Upgrades

- Add freshness SLA contract per real-time domain.
- Add stale-data refusal policy with deterministic reason codes.
- Add API outage fallback policy and bounded degraded behavior.

### 37.18 Run 16 Completion Upgrades

- Add jurisdiction model (`country`, `state/province`, `industry/sector`).
- Add regulatory trust-tier source registry requirements.
- Add compliance confidence scoring contract per claim.

### 37.19 Run 17 Completion Upgrades

- Add governance rules for trust score updates (owner + approval path).
- Add explainable trust-factor persistence in audit packet.
- Add deterministic trust override constraints (no runtime ad-hoc overrides).

### 37.20 Run 18 Completion Upgrades

- Add max-hop and max-time budgets for deep research.
- Add cycle detection and recursion-stop behavior.
- Add hop failure semantics (`retryable`, `terminal`, `partial-with-disclosure`).

### 37.21 Run 19 Completion Upgrades

- Add explicit `as_of` semantics and time-window anchoring.
- Add deterministic timeline diff algorithm specification.
- Add missing-date handling with confidence downgrade rules.

### 37.22 Run 20 Completion Upgrades

- Add risk score bands and factor-weight registry.
- Add calibration method and drift-check requirements.
- Add mandatory non-advice guard text contract in risk outputs.

### 37.23 Run 21 Completion Upgrades

- Add identity-gated internal memory merge policy.
- Add recency weighting policy for internal memory evidence.
- Add contradiction rule when internal memory and external evidence conflict.

### 37.24 Run 22 Completion Upgrades

- Add enterprise report schema validator and required-section checks.
- Add section completeness rules (`exec summary`, `findings`, `risk`, `appendix`, `citations`).
- Add export mode contract (`brief`, `standard`, `deep`, `report`) with citation parity.

## 38) Cross-Run Closure Requirements (Additive, Mandatory)

This section closes system-level completeness gaps and fulfills the wire-ready placeholders from Section `17.3`.

### 38.1 Canonical Packet Appendix Requirement

Must add one canonical packet appendix covering:
- `TurnInputPacket`
- `SearchAssistPacket`
- `ToolRequestPacket`
- `EvidencePacket`
- `SynthesisPacket`
- `WritePacket`
- `RiskPacket`
- `ComparisonPacket`
- `EnterpriseReportPacket`
- `AuditPacket`

Rule:
- each packet must include schema version, producer owner, consumer owner, required fields, and fail-closed validation behavior.

### 38.2 Canonical Reason-Code Registry Requirement

Must define one plan-wide reason-code registry with:
- unique code IDs
- owner engine
- user-visible wording contract
- retryability class
- severity class

No run may define ad-hoc reason codes outside the canonical registry.

### 38.3 Canonical Idempotency Key Table Requirement

Must define one idempotency appendix for all write paths (`PH1.F` + `PH1.J`) including:
- write path name
- key recipe
- uniqueness index
- duplicate replay behavior

No write path may ship without idempotency recipe registration.

### 38.4 Dependency DAG Requirement

Must add a run dependency DAG defining:
- hard prerequisites
- optional parallelizable runs
- blocked-on relationships

Execution rule:
- a run cannot start unless all its hard prerequisite runs are green and ledger-proven.

### 38.5 Requirement-to-Run-to-Test Trace Matrix

Must add one traceability matrix:
- requirement ID
- owning run(s)
- acceptance tests
- CI scripts
- proof command

Closure rule:
- every enterprise requirement in Section `35` must map to at least one test and one proof command.

### 38.6 Production Lock and SLO Gate Requirement

Must add one production lock section defining:
- release SLOs (`latency`, `citation coverage`, `refusal correctness`, `freshness compliance`, `risk score reproducibility`)
- hard pass/fail thresholds
- waiver authority and expiry rules

No production promotion is allowed if any lock gate is red.

## 39) Canonical End-to-End Build Program (Authoritative Run Sequence)

This section rebuilds the full plan into one executable sequence without removing any prior requirements.

Interpretation rule:
- Sections `0-38` remain the complete requirement catalog.
- This section is the authoritative implementation order for building the full system end to end.

Global run gate (applies to every run below):
- Start: `git status --porcelain` must be empty.
- End: commit + `git push origin main` + `git status --porcelain` empty + ledger proof line with exact commands/tests.

### 39.1 Phase A — Contracts and Governance Foundation

#### Run 1 — Canonical Packet Registry
- Implement and lock packet schemas (`TurnInput`, `SearchAssist`, `ToolRequest`, `Evidence`, `Synthesis`, `Write`, `Comparison`, `Risk`, `Report`, `Audit`).
- Attach schema versioning and compatibility rules.

#### Run 2 — Reason-Code Registry
- Implement one canonical reason-code registry with owner engine and fail-closed behavior.
- Remove ad-hoc reason-code drift from all lanes.

#### Run 3 — Idempotency Registry
- Implement one canonical idempotency table for all PH1.F/PH1.J write paths.
- Define duplicate replay behavior per write path.

#### Run 4 — Dependency DAG and Trace Matrix
- Implement run dependency DAG and requirement-to-run-to-test matrix.
- Block out-of-order execution in CI.

### 39.2 Phase B — Retrieval Core and Safety

#### Run 5 — Proxy Universal Layer
- Implement and lock proxy modes (`off`, `env`, `explicit`) with startup probe and safe diagnostics.

#### Run 6 — URL Fetch Core
- Implement deterministic URL fetch and extraction, MIME/redirect/charset policy, and fail-closed reasons.

#### Run 7 — Chunk/Hash/Citation Foundation
- Implement deterministic chunking, `hash_version`, collision handling, and citation binding.

#### Run 8 — Query Planning and Top-K Selector
- Implement deterministic query rewrite ladder and Top-K scoring/tie-break rules.

#### Run 9 — Web Provider Ladder
- Implement Brave lead + OpenAI fallback thresholds, budgets, and provider health state routing.

#### Run 10 — News Provider Ladder
- Implement Brave lead + GDELT assist deterministic merge with recency/date normalization.

#### Run 11 — Image/Video Retrieval Lane
- Implement deterministic image/video query and ranking, safe-search policy, and media metadata schema.

### 39.3 Phase C — Grounded Answering and Output

#### Run 12 — PH1.D Grounded Synthesis
- Implement evidence-only synthesis contract and claim-to-citation validator gate.

#### Run 13 — PH1.WRITE + PH1.TTS Output Contract
- Implement structured professional text formatting and voice parity from the same formatted payload.

#### Run 14 — Observability and Debug Packet
- Implement standard debug packet with safe fields and redaction guarantees.

#### Run 15 — Performance, Cost, and Importance Tiers
- Implement latency/cost budgets, queue/backpressure, and deterministic `low|medium|high` search-depth policy.

### 39.4 Phase D — Reliability and Adaptive Quality

#### Run 16 — Two-Tier Cache + Parallel Retrieval
- Implement L1/L2 cache contracts, deterministic fan-out, and stable merge ordering.

#### Run 17 — Quality Gates and Replay Harness
- Implement benchmark corpus, replay determinism gates, and regression thresholds.

#### Run 18 — Learning Layer
- Implement failure-signature model, bounded session-level adaptation, builder-governed promotion path, and rollback drills.

### 39.5 Phase E — Enterprise Expansion

#### Run 19 — Structured Data Connectors
- Implement registry/filings/government/academic/product/pricing/patent source adapters.

#### Run 20 — PDF, Table, and Filing-Aware Parsing
- Implement PDF extraction, table schema extraction, and filing-specific parsers.

#### Run 21 — Competitive Intelligence Mode
- Implement comparison schema, feature/pricing/entity dedup, and SWOT evidence output.

#### Run 22 — Real-Time API Mode
- Implement API-first lanes for time-sensitive domains with freshness TTL and stale-data refusal.

#### Run 23 — Regulatory and Jurisdiction Mode
- Implement jurisdiction routing, compliance confidence flags, and regulatory trust-tier filtering.

#### Run 24 — Trust/Spam/Official Source Model
- Implement trust ranking vector, official source detection, spam/clickbait suppression, and explainable trust factors.

#### Run 25 — Deep Multi-Hop Research
- Implement deterministic hop planner, max-hop/max-time budgets, cycle detection, and hop audit proofs.

#### Run 26 — Temporal Comparison Mode
- Implement `as_of` windows, timeline extraction, and deterministic change detection.

#### Run 27 — Risk Scoring Mode
- Implement factorized risk model, calibration rules, confidence coverage, and non-advice guardrails.

#### Run 28 — Internal + External Merge
- Implement memory/context merge contract with strict boundary (memory guides retrieval, never overrides evidence).

#### Run 29 — Enterprise Report Mode
- Implement long-form report templates, required sections, citation completeness validation, and export modes.

### 39.6 Phase F — Final Lock and Release

#### Run 30 — Production Lock and Final Acceptance
- Run full CI gate matrix, SLO gates, traceability checks, and final evidence pack closure.
- Enforce no-release-on-red for any mandatory gate.

### 39.7 Coverage Map to Existing Sections

Coverage guarantee:
- Runs `1-4` satisfy Section `38` closure requirements and Section `17.3` wire-ready addenda.
- Runs `5-18` satisfy core requirements in Sections `1-34` including safety, quality, and parity enhancements.
- Runs `19-29` implement all enterprise capabilities in Section `35`.
- Run `30` enforces release lock requirements in Section `38.6` and final closure expectations in Sections `12`, `15`, and `37`.

## 40) Final Wiring and Ownership Contract (Authoritative for Finished System)

This section is authoritative for end-to-end runtime wiring when the full build program is complete.

### 40.1 Top-Level Runtime Law

1. `PH1.OS` orchestrates all engine sequencing and handoffs.
2. `PH1.X` is the only runtime conversation-move authority.
3. `PH1.E` is the only read-only external retrieval execution authority.
4. `PH1.D` is synthesis-only and evidence-bounded (non-authoritative).
5. `PH1.WRITE` is presentation-only and may not change semantic truth.
6. `PH1.TTS` renders voice from PH1.WRITE formatted output only.
7. `PH1.J` is append-only audit authority.
8. `PH1.F` is persistence/projection authority.
9. No direct engine-to-engine calls; all transitions flow through orchestrated contracts.

### 40.2 Canonical Turn State Machine

Allowed turn states:
- `TURN_ACCEPTED`
- `INPUT_PARSED`
- `INTENT_CLASSIFIED`
- `PLAN_SELECTED`
- `RETRIEVAL_EXECUTED`
- `EVIDENCE_LOCKED`
- `SYNTHESIS_READY`
- `OUTPUT_RENDERED`
- `AUDIT_COMMITTED`
- `TURN_COMPLETED`
- `TURN_FAILED_CLOSED`

Allowed transitions:
- `TURN_ACCEPTED -> INPUT_PARSED -> INTENT_CLASSIFIED -> PLAN_SELECTED`
- `PLAN_SELECTED -> RETRIEVAL_EXECUTED -> EVIDENCE_LOCKED -> SYNTHESIS_READY -> OUTPUT_RENDERED -> AUDIT_COMMITTED -> TURN_COMPLETED`
- `any non-terminal state -> TURN_FAILED_CLOSED` (with reason code + proof refs)

No silent transitions:
- every transition must append one audit row in `PH1.J`.

### 40.3 Deterministic Gate Order (Must Be Enforced Every Turn)

1. Session/identity gate (`PH1.L` + `PH1.VOICE.ID` when voice lane is active).
2. Input quality gate (`PH1.C` + optional SRL/LANG repairs).
3. Intent/mode gate (`PH1.NLP`, `PH1.PRUNE`, `PH1.DIAG`, final decision by `PH1.X`).
4. Policy/access/cost gate (`PH1.POLICY`, `PH1.ACCESS`, `PH1.QUOTA`/`PH1.COST`).
5. Retrieval execution gate (`PH1.SEARCH` plan, `PH1.E` execution, proxy/provider checks).
6. Evidence integrity gate (`EvidencePacket` validation and citation traceability checks).
7. Synthesis gate (`PH1.D` evidence-only synthesis).
8. Output gate (`PH1.WRITE` formatting + optional `PH1.TTS`).
9. Audit/persistence gate (`PH1.J` append, `PH1.F` projection updates).

### 40.4 Canonical Packet Handoff Map (`producer -> packet -> consumer`)

| Producer | Packet | Consumer | Rule |
|---|---|---|---|
| `PH1.C`/`PH1.SRL`/`PH1.LANG` | `TurnInputPacket` | `PH1.NLP` / `PH1.X` | input only, no decisions |
| `PH1.NLP`/`PH1.PRUNE`/`PH1.DIAG` | `SearchAssistPacket` | `PH1.X` | advisory only |
| `PH1.X` | `ToolRequestPacket` | `PH1.SEARCH` -> `PH1.E` | authoritative dispatch intent |
| `PH1.E` | `EvidencePacket` | `PH1.D` / `PH1.WRITE` / `PH1.J` | authoritative retrieval truth |
| `PH1.D` | `SynthesisPacket` | `PH1.WRITE` | no new facts allowed |
| `PH1.WRITE` | `WritePacket` | API response + `PH1.TTS` + `PH1.J` | presentation only |
| `PH1.D`/`PH1.WRITE` (mode-specific) | `ComparisonPacket` / `RiskPacket` / `EnterpriseReportPacket` | API response + `PH1.J` | evidence-bound structured modes |
| Orchestrator | `AuditPacket` | `PH1.J` -> `PH1.F` projections | append-only audit closure |

### 40.5 Mode Flow Contracts (End-to-End)

#### 40.5.1 Standard Web/News Answer
- `PH1.X` selects retrieval mode.
- `PH1.SEARCH` generates deterministic query plan.
- `PH1.E` executes provider ladder + URL open/chunking as needed.
- `PH1.D` synthesizes from evidence only.
- `PH1.WRITE` renders final text; `PH1.TTS` speaks same structure when voice enabled.
- `PH1.J` records full proof chain.

#### 40.5.2 Structured Data Mode
- `PH1.X` selects `structured_data` mode.
- `PH1.E` calls structured connectors/parsers (tables/PDF/filings).
- output must include typed extraction rows before narrative synthesis.
- `PH1.D` can summarize typed rows but cannot bypass extracted schema.

#### 40.5.3 Competitive Intelligence Mode
- `PH1.E` retrieves multi-source competitor evidence.
- dedup + normalization (units/currencies/features) occurs before synthesis.
- output must include canonical comparison schema with source refs.

#### 40.5.4 Real-Time Mode
- API-first retrieval is mandatory.
- freshness TTL check runs before synthesis.
- stale data triggers fail-closed or explicit stale disclosure by policy.

#### 40.5.5 Regulatory/Compliance Mode
- jurisdiction resolution runs first.
- trust-tier filtering prioritizes official/regulatory sources.
- unsupported compliance claims must fail closed with explicit reason code.

#### 40.5.6 Multi-Hop Deep Research Mode
- planner emits bounded hop chain.
- each hop yields evidence refs consumed by next hop.
- final answer requires complete hop proof chain and stop-condition proofs.

#### 40.5.7 Temporal Comparison Mode
- retrieval enforces `as_of` windows.
- timeline extraction + deterministic diff generation.
- output includes changed/unchanged fields with citations.

#### 40.5.8 Risk Scoring Mode
- factor evidence retrieval (financial/legal/regulatory/news).
- deterministic risk computation with calibrated factor weights.
- output includes score, factor breakdown, confidence, and guardrail text.

#### 40.5.9 Internal + External Merge Mode
- internal memory/context retrieval is identity-gated.
- memory shapes retrieval focus only.
- external evidence remains truth authority for final claims.

#### 40.5.10 Enterprise Report Mode
- same evidence core, different output contract.
- required sections: executive summary, findings, risk, comparisons, appendix, citations.
- export modes (`brief`, `standard`, `deep`, `report`) must preserve citation parity.

### 40.6 Final Ownership Matrix (Decision, Execution, and Write Ownership)

| Responsibility | Authoritative Owner | Non-Authoritative Contributors |
|---|---|---|
| turn orchestration | `PH1.OS` | none |
| next move decision (`clarify/refuse/retrieve`) | `PH1.X` | `PH1.NLP`, `PH1.PRUNE`, `PH1.DIAG` |
| query planning hints | `PH1.SEARCH` (assist) | `PH1.KNOW`, `PH1.CACHE`, `PH1.CONTEXT` |
| external retrieval execution | `PH1.E` | none |
| policy/access/quota gating | `PH1.POLICY` + `PH1.ACCESS` + `PH1.QUOTA/COST` | `PH1.PAE` hints only |
| evidence truth | `EvidencePacket` produced by `PH1.E` | none |
| synthesis | `PH1.D` (non-authoritative) | none |
| presentation formatting | `PH1.WRITE` | none |
| voice playback | `PH1.TTS` | none |
| audit log truth | `PH1.J` | none |
| persistence/projections | `PH1.F` | none |
| learning proposals | `PH1.FEEDBACK`/`PH1.LEARN`/`PH1.PAE` | `PH1.GOV` approval lane |
| activation/promotion | `PH1.GOV` / Builder | none |

### 40.7 Failure Ownership Map (Fail-Closed)

| Failure class | Detection owner | User-facing owner | Required behavior |
|---|---|---|---|
| invalid input/session/identity | `PH1.X` with session/identity providers | `PH1.X`/adapter | refuse/clarify deterministically |
| policy/access denial | `PH1.ACCESS`/`PH1.POLICY` | `PH1.X` | fail closed; no retrieval |
| proxy/provider transport failure | `PH1.E` | `PH1.X` + `PH1.WRITE` | safe fail_detail; no invented fallback |
| no evidence / low evidence | `PH1.D` validator + policy thresholds | `PH1.WRITE` | insufficient-evidence response |
| citation mismatch | synthesis/output validator | `PH1.X` | fail closed, no answer release |
| stale real-time data | freshness gate | `PH1.X`/`PH1.WRITE` | stale disclosure or refusal per policy |
| compliance uncertainty | regulatory mode validator | `PH1.WRITE` | confidence downgrade or refusal |
| mode budget exhaustion | quota/cost gates | `PH1.X` | deterministic degraded mode |

### 40.8 End-to-End “Perfect Machine” Invariants

1. Same inputs + same policy snapshots + same evidence -> same output envelope.
2. Every factual claim has citation traceability to `sources[]` or `content_chunks[]`.
3. No synthesis outside `EvidencePacket` truth boundary.
4. No runtime governance bypass (`PH1.GOV` remains activation authority).
5. No hidden side effects in read-only research lane.
6. Every turn is replayable from `PH1.J` + `PH1.F` records.
