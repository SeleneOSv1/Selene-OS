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
`-> PH1.SEARCH` (merged `PH1.WEBINT` capability)  
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

## 4.1 PH1.WRITE Structured Output Rules

For text output, PH1.WRITE must format responses with:

- Clear headings
- Bullet points
- Explicit citations

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
- Voice output: same structure, spoken clearly in requested language

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

1. Add one canonical packet registry appendix (`TurnInput`, `SearchAssist`, `ToolRequest`, `EvidencePacket`, `SynthesisPacket`, `WritePacket`, `AuditPacket`).
2. Add one canonical ownership appendix listing authoritative engine per decision.
3. Add one canonical handoff map per run (`producer -> packet -> consumer`).
4. Add one canonical failure map (`reason_code` owner engine + fail-closed behavior).
5. Add one canonical idempotency key recipe table per write path (`PH1.F` + `PH1.J`).
6. Add one CI gate map from each run to required scripts/tests.

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
