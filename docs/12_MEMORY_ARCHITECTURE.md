# Section 2: Memory Architecture + Device Persistence (Foundational - Non-Negotiable)

## Section 2.0: Overview

Selene uses multiple memory layers with strict boundaries, plus a mandatory device persistence system. Together, these layers ensure: audit-proof history (never rewritten), correct real-time operation (editable truth), human relationship continuity (explicitly non-authoritative), accurate felt memory without being invasive, and offline resilience (encrypted storage + idempotent sync).

The core layers are:

- A) Immutable Event Ledger (append-only operational history)
- B) Current State (editable operational truth, rebuildable)
- C) Emotional Memory Threads (relationship continuity, non-authoritative)
- D) Learning Stores + Vocabulary Artifacts (accuracy/personalization, never authority)
- E) Device Persistence (Device Vault + Outbox)

Key boundary (non-negotiable):

- Conversation text is never business truth.
- Memory can improve accuracy and continuity, but it must never grant authority or execute actions.

## Section 2.1: Immutable Event Ledger (Operational History - Bank Statement)

Definition: The Immutable Event Ledger is Selene's append-only operational history. It is never edited and never deleted. It is written by deterministic simulations and core runtime events.

Used for: Audit proof, compliance evidence, dispute resolution, forensics, and traceability.

Examples:

- JD requested invite for Position P-123
- AP approved compensation override
- Yoyo accepted terms v1.2
- Wake enrollment completed
- Phase X barge-in stop-TTS triggered (reason: USER_SPEECH_DETECTED)

Hard rule: Conversation text is not business truth. Only ledger/audit events are admissible as operational proof.

## Section 2.2: Current State (Operational Truth - Bank Balance)

Definition: Current State is the editable snapshot used for real-time operation. It is derived from the Immutable Event Ledger plus the latest authoritative updates. It can be rebuilt from the ledger when needed.

Used for: Session routing and runtime decisions that require current truth; device/session configuration; non-authoritative cached preferences and pointers.

Examples:

- JD AP tier = 3
- Yoyo consent = granted
- Yoyo access engine = active
- Wake threshold = 0.61
- Last applied package_id = PKG-2026-01-17-001

Hard rule: Current State must never become a substitute for the Access Engine or simulation gating. It may cache references; it must not confer authority.

## Section 2.3: Emotional Memory Threads (Human Layer - Non-Authoritative)

Definition: Emotional Memory Threads store relationship continuity signals (tone, rituals, trust signals) to improve interaction quality. They are not business truth and never confer execution authority.

What it may store (examples):

- Tone preferences
- Conversational rituals
- Trust milestones
- Emotional milestones
- Silence patterns and pacing preferences

Rules:

- Voice-locked (no cross-user mixing)
- Must respect explicit privacy commands (forget / do not remember / recall only if asked)
- Must never influence permissions, approvals, simulations, or execution
- May decay or soften over time by policy

## Section 2.4: Learning Stores + Vocabulary Artifacts (Per-User / Per-Company / Global Derived) (Non-Negotiable)

Purpose: Selene improves accuracy and personalization while preserving strict boundaries: private per-user learning, controlled per-company shared vocabulary (non-sensitive only), and optional global improvement via derived, de-identified signals.

Hard rule: New words and corrections do not live inside LLaMA as memory. They live as Selene-owned profiles and versioned artifacts that are provided as hints to STT and understanding. Model-weight improvements occur only through the training + packaging pipeline.

### Section 2.4.1: Per-User Learning Capsule (Private - Encrypted - Voice-Locked)

Definition: A per-user encrypted profile store (device + cloud) containing bounded personalization artifacts.

May include:

- Per-user language profile (correction pairs, slang map, preferred phrasing)
- Per-user vocabulary (names, vendors, internal terms)
- Per-user STT adaptation profile (phrase hints / pronunciation hints, bounded)
- Per-user voiceprint profile references (identity support only; sourced from identity systems; not authority)
- Per-device wake tuning parameters (threshold/debounce/VAD sensitivity), per user/device

Rules:

- Voice-locked (no cross-user mixing)
- Encrypted at rest and in transit
- Non-authoritative: cannot grant permissions or execute actions

### Section 2.4.2: Per-Company Vocabulary Pack (Shared Inside One Company - Non-Sensitive Only)

Definition: A company-scoped vocabulary pack used to improve STT/understanding across members of a single company.

May include:

- Product and service names
- Approved domain terms / jargon
- Common role words (non-sensitive)

Rules:

- Non-sensitive only
- Auditable (who added, when, why)
- Must never include identity documents; private names only if explicitly permitted by policy
- Does not change permissions or authority

### Section 2.4.3: Global Derived Signals (Opt-In - De-Identified - Non-Reversible)

Definition: A global learning stream used to improve Selene overall without copying private user data.

May include:

- Aggregated token frequency statistics (derived)
- Generic slang mappings (non-identifying)
- Intent confusion patterns (derived)
- Wake/STT error statistics (derived)
- Barge-in outcomes (derived)

Rules:

- Opt-in only (consent-gated)
- No raw identity documents, no biometric data, no private names by default
- De-identified and non-reversible
- Used only for training and evaluation, never for permissions

### Section 2.4.4: Capture -> Learn -> Package -> Apply (Deterministic Flow)

Capture (during interaction):

- If Selene is uncertain, she produces a single clarification question.
- When the user corrects Selene, a correction pair is captured: what was heard -> what was meant.

Store (immediate, per-user):

- The correction pair is recorded as an immutable event and queued for sync when offline.
- The per-user language profile may update as Current State (cache) and persist deterministically.

Produce artifacts (cloud, Phase I):

- Per-user updates produce a versioned STT adaptation artifact (bounded).
- Per-company updates produce a versioned company vocabulary pack (guarded).
- Global improvement consumes derived signals only (opt-in).

Ship (packages):

- Artifacts ship via signed packages with deterministic apply order and rollback support.

Apply (device):

- Device verifies signature/hashes, applies deterministically, logs apply/rollback events, and never double-applies.

Hard rule: No learning artifact can change permissions, roles, access tiers, or execute actions. Learning improves accuracy and personalization only.

## Section 2.5: Device Vault + Outbox Ownership (Phase B Canonical)

Normative ownership:

- The canonical and authoritative mechanics for Device Vault + Outbox are owned by Engine B.
- This memory document is non-normative for storage/queue mechanics and must not redefine Engine B rules.

Memory integration points (informative only):

- Memory capsules, atoms, do-not-mention rules, and continuity deltas may be persisted via Phase B vault paths.
- Offline memory deltas must flow through Phase B outbox semantics (append-only, ack-gated deletion, idempotent replay).
- Multi-device continuity and restore behavior for memory artifacts is implemented on top of Phase B + Phase F sync.

Hard rule:

- Memory never bypasses Engine B boundaries and never upgrades authority through persistence paths.

## Section 2.6: Crash/Reboot and Replay (Reference to Engine B)

Crash/reboot durability, replay idempotence, and upload semantics are owned by Engine B.

This document only adds memory-specific expectation:

- Memory replay must not duplicate atoms, regress versions, or re-run simulations.

All failure/retry/ack behavior must follow the deterministic reason-coded model defined in Engine B.

## Section 2.7: iPhone Policy (Memory View)

iPhone-specific handling for vault/outbox also inherits Engine B policy.

Memory requirement on iPhone:

- Memory artifacts may be captured within explicit sessions and must synchronize through the same deterministic outbox discipline.

No iPhone exception may weaken Engine B invariants.

## Section 2.10: Logging + Auditing (Traceability Requirement)

All storage and queue events must be traceable via ledger/audit events, including (at minimum):

- session_id
- user_id (if known)
- event type
- outbox item_id
- retry_count
- upload timestamps
- status (success/failure with reason code)

Hard rule: Traceability is mandatory for compliance and debugging. Logs are not optional.

## Section 2.11: Memory Retrieval + Composition Contract (vNext) (Foundational)

Purpose: Define Selene's memory retrieval and composition pipeline so Selene's felt memory is accurate, deterministic, non-invasive, and privacy-safe, while preserving strict boundaries.

### Section 2.11.1: Memory Types (Operational Definition)

Working Memory (Session Continuity):

- What's in the current session (last N turns / active turn context).
- Fast, real-time memory that expires when the session ends.

External Memory (Persistent Memory):

- Persistent sources: Archive (history), ledger/audit (truth), Current State (operational truth), emotional threads (tone continuity), learning/atoms (accuracy hints).
- Enables continuity across sessions.

Trained Capability (Learning Model):

- Model weights/adapters are not memory storage.
- Training improves extraction, ranking, and summarization, but memory remains Selene-owned artifacts and stores.

### Section 2.11.2: Retrieval Pipeline (Deterministic Layers)

Layer 1: Capture (Always)

- Archive transcript (history; non-authoritative)
- Log audit/ledger events (truth; authoritative)
- Update Current State (editable operational truth)
- Update Emotional Threads (non-authoritative tone continuity)

Layer 2: Extract Memory Atoms (Deterministic)

- Extract small structured items from interactions (preferences, vocabulary, long-running projects, relationship rituals).
- Store atoms in per-user capsules (voice-locked) and optionally per-company packs when policy permits.
- Atoms never become global unless explicitly allowed and de-identified.

Layer 3: Index (Semantic Lookup)

- Store semantic keys/embeddings for atoms and bounded archive references to enable fast lookup.
- Indexing must be bounded and deterministic in limits and ordering.

Layer 4: Retrieve (Pull Only When Relevant)

- On each turn, retrieve only relevant atoms and bounded excerpts for the current request.
- Retrieval targets: relevant facts/preferences/projects, last decisions, prior context to avoid redundancy.

Layer 5: Compose (Minimal Context Bundle)

- Compose a sanitized minimal context bundle for Phase D/X (and other downstream components):
  - minimal relevant Current State facts
  - minimal relevant memory atoms
  - short bounded archive excerpts (only when needed for long-running projects)

### Section 2.11.3: Push vs Pull Discipline (Non-Invasive Memory)

Push memory (always-loaded, tiny, stable set):

- name (if permitted), primary language preference, explicit contact preference, active project IDs (if tracked), safe do-not-repeat flags.

Pull memory (retrieved only when relevant):

- everything else, including atoms, archive excerpts, and long-tail preferences.

### Section 2.11.4: Confidence Rules (Memory Accuracy Discipline)

- If a retrieved memory item is uncertain or conflicts with current context, Selene must ask one clarifying question before using it.
- Critical-token uncertainty (names/numbers/dates/amounts) must be treated as confirm/clarify before downstream action candidacy.

### Section 2.11.5: Privacy and Safety Rules (Non-Negotiable)

- Users can request forgetting; forgetting applies to memory atoms and emotional threads within policy boundaries (never rewrites audit/ledger truth).
- Memory atoms and training data never mix between users unless explicitly authorized (e.g., approved company vocab pack).
- Raw ledger dumps are never used for memory retrieval or presented. Only bounded summaries/excerpts are allowed.
- Memory must never grant authority: permissions, Access Engines, approvals, and execution remain Control-layer responsibilities.

### Section 2.11.6: Felt Memory Behavior (How This Should Feel)

- Selene pulls only relevant memory and integrates it naturally without dumping history.
- Selene avoids redundancy: she should not ask the same stable question twice unless facts changed or were never confirmed.
- Selene uses atoms to create continuity so it feels like Selene remembers everything in context, without being invasive.

### Section 2.11.7: Training's Role in Felt Memory (Phase I Linkage)

Training improves:

- memory atom extraction quality
- retrieval ranking relevance
- stable rolling summaries for long-running work
- natural phrasing of recalled items

All improvements ship via deterministic artifacts and packages. Training never becomes direct memory storage.

### Section 2.11.8: Implementation Components (Section 2 Integration Points)

This contract is implemented via three deterministic components:

- Memory Atom Extractor (creates atoms under strict rules)
- Retrieval Ranker (bounded, deterministic relevance selection)
- Context Bundle Composer (minimal sanitized bundle for Phase D/X)

These components consume and respect: ledger/current state, archive references, per-user capsule artifacts, emotional threads, and privacy policies.

### Section 2.11.9: Retrieval Budgets + Ranking Rules (Deterministic)

Purpose: Selene must retrieve memory in a way that feels natural and human, while staying bounded, deterministic, and non-invasive. This section defines strict budgets and deterministic ranking rules so memory recall is consistent and never becomes a dump.

Retrieval budgets (hard caps, per turn):

- Context bundle size cap: maximum 32 KB total payload passed downstream for memory assist.
- Memory atoms cap: maximum 20 memory atoms per turn.
- Archive excerpt cap: maximum 2 excerpts per turn; each excerpt is bounded and short (no full history dumps).
- Always-loaded (push) memory cap: must remain tiny and stable (name, language preference, active project IDs, do-not-repeat flags).
- Pull memory cap: everything else is retrieved only when relevant and must fit inside the bundle caps above.

Deterministic ranking signals (bounded): Memory candidates are ranked using deterministic signals only:

- Pinned > unpinned
- Verified > user-stated > tool-derived > inferred (provenance priority)
- Higher confidence > lower confidence
- More recent use > older use
- Higher use_count > lower use_count

Deterministic ordering (tie-breakers): If candidates tie on ranking signals, Selene must resolve ties deterministically in this exact order:

- Higher provenance priority
- Higher confidence
- More recent last_used timestamp
- Higher use_count
- Stable lexicographic ID order (final tie-break)

Hot vs cold memory rule (human-feel without being invasive):

- Hot memory is the always-loaded tiny set (push memory). It is always available and should cover stable preferences to prevent repeated questions.
- Cold memory is everything else (pull memory). It is retrieved only when relevant and must always respect the budgets above.

Hard rule: Selene must never exceed the retrieval caps. If the relevant set is too large, Selene must return the highest-ranked items only and ignore the rest (no partial dumps, no best effort overflow).

### Section 2.11.10: Freshness + Conflict Handling (Deterministic)

Purpose: Selene must treat memory as helpful context, not truth. This section defines deterministic freshness rules and conflict handling so Selene never uses stale or contradictory memory as if it is certain.

Freshness priority (deterministic order): When multiple memory items relate to the same topic, Selene must prefer items in this order:

- Pinned items (explicitly preserved)
- Verified items (confirmed by the user, admin, or authoritative process)
- More recent items (higher last_confirmed_at or last_used_at)
- Higher confidence items (confidence_bp)
- Higher provenance priority (verified > user-stated > tool-derived > inferred)

Staleness rule (bounded, deterministic): Memory atoms may become stale. Selene must treat an atom as stale if:

- It has not been confirmed/used within a defined freshness window (policy-defined per atom kind), or
- A newer conflicting atom exists for the same key/topic.

Conflict detection (hard rule): A conflict exists if any of the following disagree:

- Current State facts (operational truth)
- Verified memory atoms (confirmed facts/preferences)
- Archive excerpts (non-authoritative history)
- Ledger-derived summaries (truth evidence summaries, never raw dumps)

Conflict handling outcome (deterministic): If a conflict is detected, Selene must do exactly one of the following (in order):

- If Current State is authoritative for the topic: treat memory as non-authoritative and do not override Current State.
- If memory is stale or confidence is below threshold: mark the memory as uncertain and request one clarification before using it.
- If two memory atoms conflict and both appear plausible: request one clarification that resolves the single missing/ambiguous detail.

One-question rule (non-negotiable): When memory is uncertain or conflicting, Selene must ask exactly one clarifying question before relying on it. If the user does not clarify, Selene must proceed without using the uncertain memory item.

No silent overwrites (hard rule): Selene must never silently replace a stable preference or fact based on a single ambiguous turn. Updates to memory atoms must be:

- deterministic (reason-coded)
- bounded (size and frequency)
- confidence-aware (requires sufficient confidence or explicit user confirmation)

Output tagging (required): The Context Bundle Composer must tag memory items as one of:

- CONFIRMED (safe to use)
- TENTATIVE (may be relevant but uncertain)
- STALE (old; do not rely on without confirmation)
- CONFLICT (contradicts another source; must clarify)

Only CONFIRMED items may be used as strong context. TENTATIVE/STALE/CONFLICT items must trigger the one-question rule if they matter to correctness.

### Section 2.11.11: User Control + Non-Invasive Transparency (Mandatory)

Purpose: Selene's memory must feel human and helpful, not invasive. The user must be able to control what is remembered and how it is recalled, and Selene must avoid creepy recall behavior.

User control commands (required, deterministic effects):

- "What do you remember about me?"
  - Selene returns a bounded summary of SAFE_TO_SPEAK / SAFE_TO_TEXT memory atoms only.
  - INTERNAL_ONLY items are never shown.
- "Forget that" / "Delete that memory"
  - Applies to memory atoms + emotional threads within policy boundaries.
  - Never rewrites Immutable Event Ledger / Audit truth.
- "Don't remember this"
  - Prevents creation of new atoms for that topic/key (do-not-store flag).
- "Only recall this if I ask"
  - Marks an atom as pull-only (never pushed, never proactively surfaced).
- "Stop mentioning this"
  - Creates/updates a DO_NOT_MENTION rule that suppresses recall/surfacing.

Non-invasive transparency rule (how Selene should sound):

- If Selene uses memory to answer, she may provide a short, optional attribution that is not a dump, for example:
  - From what you told me last time...
  - Based on your saved preference...
- Attribution is never required, and must be omitted if it risks being invasive.

Exposure and safety discipline (hard rules):

- INTERNAL_ONLY memory items must never be placed into ContextBundle or spoken.
- SAFE_TO_TEXT items must not be spoken unless the user explicitly asks or the user's contact preference allows it.
- DO_NOT_MENTION must suppress any attempt to surface the related key/topic.

Hard rule: User control commands must be honored deterministically, logged as reason-coded memory actions, and applied consistently across devices (via Outbox + cloud mirror).

### Section 2.11.12: Redundancy Prevention Rules (Do Not Ask Twice)

Purpose: Selene must not feel robotic. Once something stable is known, Selene should not repeatedly ask it again unless there is a valid reason.

Do-not-repeat flags (required):

Selene must maintain deterministic do-not-repeat markers for stable facts and preferences, including:

- name (if permitted)
- primary language preference
- contact preference (speak/text)
- active project identifiers (if tracked)
- onboarding-stable company/user basics (where applicable)

When Selene is allowed to re-ask (strict, deterministic): Selene may re-ask a previously known item only if one of the following is true:

- Conflict detected (Current State vs memory vs new user statement)
- Stale memory (past freshness window for that atom kind)
- Low confidence (below threshold)
- User explicitly asks to review or change it
- Device restore occurred and Selene must confirm one missing critical preference (bounded to one question)

Question suppression rule (hard rule):

- If Selene asked a clarification question about a memory item and the user did not answer, Selene must not repeat it immediately.
- Instead, Selene must proceed safely without using that uncertain memory and mark it as unresolved for later (bounded).

Hard rule: Redundancy prevention must not reduce correctness. If correctness depends on a missing or conflicting memory item, Selene must ask exactly one clarifying question.

## Section 2.12: Memory Quality Measurement + Proof (Deterministic)

Purpose: World-class memory is proven, not claimed. Selene must measure memory quality deterministically and prove improvements over time without becoming invasive or unreliable.

Core metrics (minimum set):

- Wrong-recall rate: how often recalled memory was incorrect (confirmed by user correction).
- Redundancy rate: how often Selene re-asked something already known.
- Retrieval precision: fraction of retrieved items that were actually used/helpful.
- Retrieval recall (bounded proxy): how often Selene failed to retrieve an item that would have prevented a repeat question or mistake.
- Memory-caused correction rate: user corrections triggered specifically by memory usage.
- Invasiveness triggers: count of unexpected personal recall events that caused user discomfort or explicit do not remember commands.

Deterministic evaluation harness (required): Use fixed fixtures (stable inputs and expected outputs) to validate:

- bounded retrieval behavior (caps respected)
- deterministic ordering and tie-breakers
- conflict handling behavior (one-question rule)
- DO_NOT_MENTION suppression behavior
- no INTERNAL_ONLY leakage into ContextBundle

Results must be stable across runs for identical inputs (no clock randomness inside scoring).

Logging for measurement (required, bounded):

- context_bundle_bytes (bounded)
- atoms_selected_count
- excerpts_selected_count
- confirmed_vs_tentative_counts
- conflict_trigger_count
- clarification_due_to_memory_count
- do_not_mention_hits_count

Hard rule: Metrics and proofs measure memory quality only. They do not grant authority and must not affect permissions or execution behavior.

## Section 2.13: Multi-Device Restore Guarantee (Loss/Theft Recovery)

Purpose: Selene must preserve continuity if a device is lost, stolen, or replaced. A user should be able to log into a new device and recover their remembered preferences and continuity artifacts without losing one word of permitted memory.

What must be recoverable on a new device (minimum):

- Per-user learning capsule artifacts (language profile, vocabulary, pronunciation hints, correction pairs as atoms)
- Memory atoms (voice-locked) and their metadata (confidence, provenance, pinned flags, do-not-mention rules)
- Emotional memory threads (within policy and exposure rules)
- Current State snapshot pointers (non-authoritative; rebuildable)
- Package/version history pointers (last applied package_id, rollback pointers)
- Do-not-repeat flags and stable preferences

Cloud mirror guarantee (required):

- Continuity-critical artifacts stored in the Device Vault must be mirrored to the cloud as encrypted artifacts.
- Cloud mirror must support deterministic restore on a new device after re-authentication and consent/retention checks.
- Device-bound keys remain device-bound; cloud mirror uses cloud-side encryption and access control so restored data is never plaintext at rest.

Outbox sync guarantee (required):

- Any continuity-critical update must be queued through Outbox when offline and flushed deterministically when online.
- Outbox replay is idempotent: no double-apply, no duplicate memory atoms, no duplicate state transitions.

Restore behavior (deterministic, non-authoritative): When a user activates a new device:

- Restore latest cloud-mirrored continuity set (bounded, encrypted).
- Apply missing packages deterministically (verify -> apply order -> rollback if needed).
- Pull and apply any missing deltas/events idempotently (no duplicate processing).
- Resume normal operation without silently executing actions.

Hard rule: Restore is continuity recovery only. It must never execute business actions, grant permissions, or bypass simulations.

## Section 2.14: Runtime Integration (Memory Runs Every Turn)

Purpose: Section 2 is not only storage. It defines how Selene uses memory during live interaction so memory feels real, stays bounded, and never becomes invasive or unsafe.

Where memory runs (per turn, deterministic):

- Before Phase C (STT): Selene may produce a bounded Hint Bundle (language + vocabulary + pronunciation hints) to improve hearing accuracy.
- Before Phase D / Phase X (Understanding + response shaping): Selene must produce a bounded Context Bundle (minimal relevant facts + minimal atoms + optional short excerpts).
- In parallel with Emotional Engine: Selene emits bounded EmotionalSignals to EMO, which produces ToneGuidance (tone only; never facts; never authority).

Required bundles (bounded outputs):

- Hint Bundle (Phase C input): per-user vocab + per-company vocab (non-sensitive) + pronunciation hints + language preferences (bounded).
- Context Bundle (Phase D/X input): minimal Current State facts + minimal Memory Atoms + bounded archive excerpts (bounded).
- EmotionalSignals (EMO input): tone-relevant signals only; no business facts.

Hard rule:

- No INTERNAL_ONLY memory is allowed into bundles that can reach Understanding or speech output.
- If memory cannot be retrieved safely (missing consent/policy denies/over budget), Selene must fail-closed to an empty bundle and proceed safely.

## Section 2.15: World-Class Proof Requirements (Memory Is Proven, Not Claimed)

Purpose: Section 2 is world class only when the behavior is proven by deterministic tests and recorded in the Ledger. The architecture text alone is not proof.

What must be proven (minimum):

- Bounded recall discipline
- Retrieval budgets are enforced every turn (atom caps, excerpt caps, context bundle byte caps).
- Deterministic ordering and tie-breakers produce stable results.
- Freshness + conflict correctness
- Conflicting/stale memory triggers exactly one clarifying question when correctness depends on it.
- Current State is never overridden by memory.
- No invasiveness / no leakage
- DO_NOT_MENTION rules suppress recall reliably.
- INTERNAL_ONLY never reaches Context Bundle, never reaches speech, never reaches tools/sims.
- "What do you remember about me?" returns only bounded safe items.
- Redundancy prevention works
- Selene does not repeatedly ask stable facts unless conflict/staleness/low-confidence requires it.
- Multi-device restore is real
- New device restore recovers the permitted continuity set deterministically (no loss, no duplication, no silent execution).
- Outbox replay is idempotent under offline/outage/dup-ack.
- Measurement exists and is stable
- Deterministic metrics exist for wrong-recall, redundancy, retrieval precision proxy, memory-caused corrections, and invasiveness triggers.
- Metrics are stable across runs for identical fixtures.

Hard rule: No best in the world claim is allowed unless these behaviors are proven by deterministic acceptance tests and recorded as verified rows in SELENE_PHASE_COMPLETION_LEDGER.md.

## Section 2.16: Proof Reference (Ledger)

Purpose: This section records where memory-related proof is tracked. Completion remains defined only by the Ledger.

Ledger linkage (memory spans multiple proofs): Memory is proven across multiple ledger rows because it spans:

- Device persistence and idempotence (Vault + Outbox)
- Cloud durability and sync correctness
- Archive vs audit boundaries
- Per-user memory retrieval (atoms/index/bundles)
- Quality measurement harness for memory

Hard rule: If memory behavior is expanded (new retrieval rules, new privacy rules, new restore semantics, new metrics), it requires a new versioned proof row and a new deterministic acceptance test recorded in the Ledger.

## Section 2.17: Memory Track Q12–Q18 (Quality Gates + Build Plan)

Purpose: Define the memory-specific Q12–Q18 gates that raise Selene’s memory quality to world‑class standards. These are memory-track gates and must be proven via deterministic acceptance tests and recorded in `SELENE_PHASE_COMPLETION_LEDGER.md`.

Note on naming: If global Q numbers already exist, ledger rows must use a memory suffix (e.g., `Q12-MEM`) to avoid collisions while preserving the gate order.

### Q12 — Deterministic Memory Bundle + Retrieval Discipline (Defined)

Goal: Prove the Context Bundle composer, retrieval caps, and deterministic ordering work every turn.

Requirements:

- Retrieval caps enforced (bundle bytes, atoms, excerpts, push vs pull limits).
- Deterministic ordering and tie‑breakers as defined in Section 2.11.9.
- Memory items tagged correctly: CONFIRMED / TENTATIVE / STALE / CONFLICT.
- One‑question rule enforced when correctness depends on uncertain memory.
- Zero INTERNAL_ONLY leakage into Context Bundle or speech.

Acceptance test stubs (deterministic fixtures):

- AT_Q12_MEM_BUNDLE_BUDGETS
- AT_Q12_MEM_ORDERING_TIEBREAK
- AT_Q12_MEM_TAGGING
- AT_Q12_MEM_ONE_QUESTION_RULE
- AT_Q12_MEM_NO_INTERNAL_LEAK

### Q13 — Memory Update Safety + Correction Pairs (Defined)

Goal: Deterministic capture of corrections, bounded updates, and no silent overwrites.

Exact behavior to prove:

- Capture correction pairs deterministically from user corrections: `heard -> meant`.
- Create or update memory atoms with reason codes and provenance.
- No silent overwrite of critical tokens (names, numbers, dates, amounts).
- Deterministic confidence updates using fixed deltas per reason.
- Freshness windows per atom kind; stale when window exceeded.

Strict pass/fail criteria:

- PASS if every correction pair produces exactly one atom update with a reason code and deterministic confidence delta.
- PASS if critical tokens are never overwritten without explicit user confirmation or verified source.
- FAIL if any correction is dropped, duplicated, or changes multiple atoms without reason codes.
- FAIL if confidence changes are non‑deterministic or time‑based.

Constraints:

- Privacy: correction artifacts are per‑user; no cross‑user leakage.
- Latency: correction capture completes within the same turn (no extra user‑visible delay).
- Budget caps: per turn, max 5 new atoms and max 5 updates.

Acceptance test stubs:

- AT_Q13_MEM_CORRECTION_PAIR_CAPTURE
- AT_Q13_MEM_CRITICAL_TOKEN_CONFIRM
- AT_Q13_MEM_CONFIDENCE_DELTAS
- AT_Q13_MEM_NO_SILENT_OVERWRITE

### Q14 — Redundancy Prevention + Do‑Not‑Repeat (Defined)

Goal: Eliminate repeated questions once stable facts are known unless conflict/stale/low‑confidence.

Exact behavior to prove:

- Maintain deterministic do‑not‑repeat flags for stable facts (name, language, contact preference, project IDs, onboarding‑stable basics).
- Re‑ask allowed only on conflict, staleness, low confidence, user request, or device restore.
- If user ignores a clarification, mark unresolved and do not repeat immediately.

Strict pass/fail criteria:

- PASS if fixtures show no repeated questions when stable facts are confirmed.
- PASS if re‑asks occur only under allowed conditions with reason codes.
- FAIL if Selene repeats a stable question without a valid reason.
- FAIL if unresolved questions are repeated in the same session.

Constraints:

- Privacy: suppression only; no additional exposure.
- Latency: suppression decisions are O(1) lookup.
- Budget caps: do‑not‑repeat set limited to 128 keys per user; LRU eviction allowed with reason code.

Acceptance test stubs:

- AT_Q14_MEM_DO_NOT_REPEAT
- AT_Q14_MEM_REASK_GUARDS
- AT_Q14_MEM_UNRESOLVED_SUPPRESS

### Q15 — Multi‑Device Restore + Idempotent Memory Sync (Defined)

Goal: Proven restore of memory atoms and continuity artifacts across devices.

Exact behavior to prove:

- Restore continuity artifacts deterministically (atoms + metadata, emotional threads, do‑not‑mention, do‑not‑repeat, learning capsule pointers).
- Outbox replay is idempotent; no duplicate atoms or version regressions.
- Restore never executes actions; continuity recovery only.

Strict pass/fail criteria:

- PASS if restore reproduces the exact expected continuity set from fixtures.
- PASS if replay produces zero duplicates under simulated retries and dup‑acks.
- FAIL if any atom is duplicated, missing, or altered without reason code.
- FAIL if restore triggers any action or permission change.

Constraints:

- Privacy: encrypted artifacts only; device keys remain device‑bound.
- Latency: restore completes within 10 seconds for 10k atoms (fixture target).
- Budget caps: restore must not exceed 32 MB memory bundle on load.

Acceptance test stubs:

- AT_Q15_MEM_RESTORE_CONTINUITY
- AT_Q15_MEM_OUTBOX_IDEMPOTENCE
- AT_Q15_MEM_NO_ACTION_ON_RESTORE

### Q16 — Privacy + Exposure Control (Defined)

Goal: Deterministic enforcement of SAFE_TO_SPEAK / SAFE_TO_TEXT / INTERNAL_ONLY and DO_NOT_MENTION.

Exact behavior to prove:

- Enforce exposure levels for every memory item.
- DO_NOT_MENTION suppresses recall in responses and Context Bundles.
- “Forget/Don’t remember” deletes atoms/emotional threads within policy; ledger unchanged.
- “What do you remember about me?” returns only bounded safe summaries.

Strict pass/fail criteria:

- PASS if INTERNAL_ONLY never reaches Context Bundle or speech (fixtures).
- PASS if DO_NOT_MENTION suppresses all mentions in tools and speech.
- PASS if forget commands delete eligible atoms and are logged.
- FAIL if any unsafe item leaks into response or bundles.

Constraints:

- Privacy: strict; no exceptions.
- Latency: exposure filtering sub‑millisecond on 1k atoms.
- Budget caps: recall summary limited to 1 KB and 10 items.

Acceptance test stubs:

- AT_Q16_MEM_EXPOSURE_FILTER
- AT_Q16_MEM_DO_NOT_MENTION
- AT_Q16_MEM_FORGET_COMMANDS
- AT_Q16_MEM_SAFE_RECALL_SUMMARY

### Q17 — Memory Quality Measurement Harness (Defined)

Goal: Deterministic metrics pipeline that proves improvements without being invasive.

Exact behavior to prove:

- Deterministic metrics per fixture: wrong‑recall, redundancy, retrieval precision, memory‑caused corrections, invasiveness triggers.
- Metrics logged as non‑authoritative measurement events.
- Metrics never affect permissions or execution.

Strict pass/fail criteria:

- PASS if metrics are identical across runs for identical fixtures.
- PASS if every metric is produced with expected counts.
- FAIL if metrics differ between runs or are missing.

Constraints:

- Privacy: metrics aggregate and non‑identifying.
- Latency: measurement overhead < 3% per turn.
- Budget caps: per‑turn measurement log ≤ 1 KB.

Acceptance test stubs:

- AT_Q17_MEM_METRICS_STABLE
- AT_Q17_MEM_METRICS_COUNTS
- AT_Q17_MEM_METRICS_NONAUTHORITATIVE

### Q18 — Emotional Continuity Boundaries (Defined)

Goal: Emotional memory improves tone without leaking facts or affecting authority.

Exact behavior to prove:

- Emotional memory threads influence tone only; never facts or authority.
- Emotional signals are bounded and tagged; no data‑bearing facts allowed.
- Emotional memory can be forgotten independently of ledger truth.

Strict pass/fail criteria:

- PASS if emotional signals are present but never appear in factual bundles.
- PASS if tone changes are observable without changing facts/decisions.
- FAIL if emotional memory changes any content decision or action.

Constraints:

- Privacy: emotional data is voice‑locked; no cross‑user mixing.
- Latency: emotional extraction under 20 ms per turn.
- Budget caps: emotional signal payload ≤ 512 bytes.

Acceptance test stubs:

- AT_Q18_MEM_EMO_TONE_ONLY
- AT_Q18_MEM_EMO_NO_FACTS
- AT_Q18_MEM_EMO_FORGET

## Section 2.18: THREAD MEMORY STORE (vNext++)

Purpose: Keep long-lived topic continuity without replaying entire transcripts.

Thread definition (deterministic container):
- `thread_id`
- `thread_title` (short topic label)
- `key_entities` (bounded list)
- `last_decisions` (bounded list)
- `unresolved_questions` (bounded list)
- `next_step` (bounded short text)
- `evidence_pointers` (bounded list of `conversation_turn_id` refs only; never raw transcript dumps)

Session Digest:
- On `SOFT_CLOSED`/`CLOSED`, Selene emits a deterministic session digest update into the thread store.
- Digests contain summary fields + evidence pointers, not raw transcript text.

Retention defaults:
- default retention: long-window policy (tenant/user policy defined)
- pinned threads: no expiry until explicit user forget
- unresolved threads: retained until resolved (or policy window timeout)

Cross-session continuity:
- recent same-day threads and unresolved threads receive deterministic resume priority.

## Section 2.19: Memory graph Index (vNext++)

Purpose: Provide fast retrieval structure for continuity. The graph is an index, not truth.

Graph model (bounded):
- nodes: `entity | project | vendor | decision | thread`
- edges: `mentioned_with | depends_on | decided_in | blocked_by`

Graph build sources:
- memory atoms
- thread digests
- selected archive pointers

Deterministic graph retrieval:
- expansion depth and fan-out are bounded by policy
- no unbounded traversal
- stable ordering/tie-breakers: confidence, recency, use_count, then lexicographic ID

Hard rule:
- graph index never overrides Current State or audit truth.

## Section 2.20: Virtual Memory paging Rules (vNext++)

Purpose: Deliver “remembers almost everything” feel via bounded page-in behavior.

Model:
- archive acts as durable “disk”
- Context Bundle acts as bounded “RAM”
- paging selects a small deterministic set to page-in

Hard caps (unchanged):
- Context Bundle <= 32 KB
- memory atoms <= 20
- archive excerpts <= 2

Deterministic page-in order:
1. thread digest summary first
2. then 1-2 most relevant excerpts by pointer if needed
3. never dump full history

Hard rule:
- paging can improve context only; it cannot expand authority or bypass control gates.

## Section 2.21: auto-resume Policy (vNext++)

Trigger:
- when a new session opens and identity is verified (`speaker_assertion_ok` or signed-in user)

Behavior:
- Selene loads a `Resume Bundle` from the most recent active thread set.
- recency preference: threads updated within the last 72 hours first.
- unresolved threads get deterministic priority boost.
- Core rule (non-negotiable): Selene OS MUST always build/load the internal Resume Bundle on every session open when identity is verified. The tier rules below control only user-facing surfacing (auto-speak vs ask vs silent), not whether the bundle is built.

User experience:
- default resume output is short and non-invasive: “Where we left off…” in 1-3 bullets.
- user overrides:
  - “don’t resume automatically”
  - “only resume if I ask”

Hard rule:
- if identity is unknown, auto-resume is blocked.

### Section 2.21A: Retention/Resume Tiers (Deterministic)

Internal load rule (non-negotiable)
- On every session open with identity verified, Selene OS MUST build/load an internal Resume Bundle candidate set (thread digests + bounded pointers). This happens even when nothing is surfaced to the user.
- The tier rules below control ONLY user-facing surfacing (auto-speak vs ask vs silent). They do not control internal loading.

Resume tiers (user-facing surfacing)

- HOT window: 72h
  - Auto-speak is permitted (and only permitted) in HOT.
  - If voice delivery is allowed by policy and the user has not disabled auto-resume, Selene may auto-speak a short “Where we left off…” summary (1–3 bullets) for the top candidate thread.
  - If voice delivery is not allowed, the same 1–3 bullet summary is delivered to the Selene App thread as text only.

- WARM window: 30d
  - Auto-speak is forbidden.
  - Selene asks exactly one question:
    - “Do you want to resume <thread_title> or start something new?”
  - If the user chooses resume, Selene then delivers the same 1–3 bullet summary (text/voice per policy).

- COLD window: forever (policy-defined retention; default long)
  - Auto-speak and auto-suggest are forbidden.
  - Selene must not surface the thread unless the user explicitly asks (e.g., “resume the thread about <topic>”) or the current request directly depends on it and PH1.X requests a confirmation/clarification.

Pinned threads
- Pinned threads never expire until explicitly forgotten.
- Pinned affects candidate ranking (below) but does not override the tier’s surfacing rules.

Unresolved threads
- Unresolved threads are eligible until resolved OR 90 days (whichever comes first).
- After 90 days, unresolved threads decay to COLD surfacing rules.

Top candidate ranking (deterministic)
- Pinned > Unresolved > Most recent last_used_at > Highest use_count > Lexicographic ID tie-break

### Section 2.21B: Pending Work Continuity Policy (WorkOrder-aware)

Source of truth:
- WorkOrder state is authoritative for pending work continuity; memory only assists recall/suppression.

On session open (identity OK):
- if any WorkOrders with status in `{DRAFT, CLARIFY, CONFIRM}` exist within the `WARM` window (30 days),
  Selene offers resume of the top pending WorkOrder.
- if a pending WorkOrder is active inside the `HOT` window (72h), Selene may auto-speak a 1–3 bullet resume line (HOT only); otherwise Selene suggests via the single WARM question.
- if only `WARM`, Selene asks one resume question (no auto-load).

Suppression command behavior:
- if user says “forget that” about a pending WorkOrder or thread:
  - the memory system sets `DO_NOT_MENTION` for target `work_order_id` and/or `thread_id`.

Cancel command behavior:
- if user says “cancel that” about a pending WorkOrder:
  - the orchestrator marks the WorkOrder status as `CANCELED` under the canonical WorkOrder contract.
  - the WorkOrder ledger records a reason-coded status-change event.

Hard rule:
- pending work continuity must not execute actions automatically; it only resumes clarify/confirm flow.

## Section 2.22: “Remember Everything” User Preference (vNext++)

Preference field:
- `memory_retention_mode = DEFAULT | REMEMBER_EVERYTHING`

Deterministic effects:
- `DEFAULT`: standard pull-first recall (safe push memory only)
- `REMEMBER_EVERYTHING`: stronger auto-resume priority, longer retention windows, richer thread digests

Still bounded:
- per-turn bundle caps remain unchanged
- no full-history dumps

User controls (deterministic commands):
- “Remember everything going forward.”
- “Only recall if I ask.”
- “Forget this thread.”
- “Resume the thread about <topic>.”

Hard rule:
- `REMEMBER_EVERYTHING` changes retention/resume policy only; it never grants authority or executes actions.

Section Integration Note
- Memory composition consumes normalized understanding outputs and identity context from upstream systems.
- If required context is missing or confidence is insufficient, memory usage remains clarification-first and non-authoritative.
- Detailed engine contracts, schemas, and capability wiring are defined only in canonical engine contract documents.
