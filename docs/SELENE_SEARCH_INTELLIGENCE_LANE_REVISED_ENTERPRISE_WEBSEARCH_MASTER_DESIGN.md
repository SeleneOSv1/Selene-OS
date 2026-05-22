Selene Search Intelligence Lane — Revised Enterprise Websearch Master Design

DOCUMENT TYPE:
DEDICATED MASTER DESIGN / PH1.E SEARCH INTELLIGENCE LANE

TASK:
SELENE_SEARCH_INTELLIGENCE_LANE_REVISED_ENTERPRISE_WEBSEARCH_MASTER_DESIGN

BUILD CLASS:
ARCHITECTURE / SEARCH INTELLIGENCE / OPENAI-ASSISTED PUBLIC ANSWER STACK

CONTROLLING DOCUMENTS:
1. AGENTS.md
2. Selene Master Architecture Build Set
3. Selene Final Overall Architecture Build Plan
4. Selene Overall Repo-Truth Activation Pack
5. Selene Function Stack Architecture — Intent and Enterprise Stack Map
6. Selene Global Human Conversation Spine Master Architecture
7. Selene Identity + Access + Authority Spine Master Architecture
8. Selene PH1.M Human Memory Core Master Design

PURPOSE:
Define Selene’s enterprise-grade Search Intelligence Lane so public websearch, entity lookup, source-backed answers, visual search, time/weather-style public tool answers, and research-style responses feel human, useful, visual, source-backed, and at least ChatGPT-level in presentation quality while preserving Selene-owned validation, provider governance, evidence, protected execution boundaries, and audit.

0. Master Standard

Selene websearch must not be a shallow search wrapper.

Selene search must feel like a capable human research assistant using the best available intelligence, not like a provider result dump.

The target is:

OpenAI-assisted human search reasoning
+ Selene-owned evidence validation
+ best available answer behavior
+ clean ChatGPT-level or better presentation
+ source chips
+ relevant photos / visual cards where approved
+ TTS-safe natural language
+ enterprise trace and audit
+ protected fail-closed execution

The goal is not low-quality low-cost search.

The goal is:

find the best available answer
explain it naturally
show the sources cleanly
show relevant visuals when appropriate
avoid weak or wrong-source claims
never dump raw provider sludge on the user

Cost is governed. Quality is the standard.

1. Core Principles

1.1 Search should be used when it helps the user

Selene should not search every prompt blindly.

But when search is needed, Selene should use the strongest lawful search path available for the task.

Stable explanation / writing / draft / transformation
→ usually no search
→ GPT-5.5 / PH1.WRITE answer path

Current facts / latest info / source request / entity lookup / claim verification / public research
→ search needed
→ provider governance
→ best-fit search path
→ evidence validation
→ beautiful sourced answer

No-search is for speed and relevance, not cheapness.

Search is used when it improves correctness, freshness, trust, or user value.

1.2 Best available answer, not dead-end refusal

Selene must not end normal public search with a robotic dead end like:

I cannot verify.

Instead, Selene must provide the best available helpful answer within evidence and safety limits.

Correct patterns:

Best available from the sources I found...
The strongest source I found says...
The closest reliable result is...
I found evidence for X, but not for Y.
The sources support this narrower claim...
I found conflicting sources, so the safest reading is...

Important distinction:

Always answer helpfully.
Never fabricate proof.
Never present unsupported claims as verified facts.

If exact requested evidence is missing, Selene should still help:

return closest reliable result
explain what is supported
show accepted sources
suggest a next search direction
ask one useful clarification only when necessary

1.3 OpenAI should be used wherever it improves search intelligence

OpenAI / GPT-5.5 may assist with:

semantic intent understanding
query planning
entity disambiguation
search strategy
source reading assistance
evidence extraction assistance
claim extraction
contradiction comparison
summary synthesis
image relevance reasoning
answer drafting
source explanation
follow-up suggestions
same-language presentation
TTS-safe wording

OpenAI must not own:

source acceptance
claim truth
provider budget authority
protected execution
authority
business mutation
final audit truth
Desktop rendering authority
Adapter rewrite authority

The rule remains:

OpenAI helps Selene search and explain.
Selene validates what may be trusted, shown, spoken, and audited.

1.4 Same or better than ChatGPT is the minimum UX standard

For public websearch answers, the baseline user experience is:

direct answer first
natural human explanation
small clickable source chips
relevant photos / image cards when appropriate and approved
short by default
expandable when user asks
same-language answer
TTS-safe spoken answer
no raw provider JSON
no source dump
no debug clutter

If ChatGPT would answer with a clean direct answer and sources, Selene must not answer with a raw provider blob wearing a trench coat.

1.5 Public search is not protected execution

Public websearch is read-only public answer work.

It does not require simulation authority.

Protected business execution still requires authority + simulation.

Examples:

Search latest payroll rules
→ public read-only search may proceed where policy allows

Approve payroll for Tim
→ protected execution
→ authority + simulation required

Search payroll rules and approve payroll for Tim
→ search part may proceed
→ approval part must fail closed unless authority + approved simulation exist

PH1.WRITE must not turn a protected denial into implied approval.

2. Final Target Flow

Target flow:

User prompt
→ SemanticInterpreterProvider / GPT-5.5 proposes user meaning where applicable
→ PH1.X validates lane, target, risk, and owner
→ PH1.N or current entity/language extraction owner extracts entity, claim type, language, freshness
→ PH1.SEARCH or current search planning owner builds SearchPlanPacket
→ Provider Governance checks model/provider policy, budget, data-egress, provider-off/fake-provider state
→ cache / recent accepted evidence check where lawful
→ PH1.E routes to best-fit provider path
→ search provider candidates return
→ safe page read where needed and allowed
→ evidence extraction
→ entity match scoring
→ source ranking
→ wrong-source rejection
→ claim verification
→ optional visual/image metadata validation
→ AnswerPacket
→ PH1.WRITE final presentation
→ Adapter transports response packets only
→ Desktop/iPhone render answer, source chips, images/cards only from approved packets
→ TTS speaks approved final tts_text only
→ audit trace records plan, sources, decisions, costs, and final answer

This flow must remain probabilistic-first for human meaning and presentation, but deterministic-gated for evidence, provider governance, source acceptance, protected action boundaries, and audit.

3. Engine Responsibilities

3.1 PH1.X — Conversation and Lane Orchestrator

PH1.X validates the user-facing lane:

no-search public answer
websearch answer
time/weather/public tool answer
file/tool answer
private/memory-gated answer
mixed public + protected request
protected fail-closed answer
simulation-authorized execution

PH1.X must use semantic proposal where applicable and must not become a keyword router.

PH1.X must not be the final long-term formatter.

PH1.X sends structured directives to the correct owner.

3.2 SemanticInterpreterProvider / GPT-5.5

GPT-5.5 may propose:

search_required
freshness_required
source_backed_answer_required
entity_lookup
claim_verification
comparison
latest/current info
visual result useful
clarification needed
public/protected split

GPT-5.5 must not decide source truth, access, authority, protected execution, or final answer acceptance.

3.3 PH1.N or Current Entity/Language Extraction Owner

This owner extracts:

intent
language
requested entity
normalized entity
claim type
role target
freshness requirement
public/protected boundary

It must preserve both:

user’s actual wording
normalized entity/query representation

Example:

Captured text: Who is the CEO of Moonridge Organic Wines?
Requested entity: Moonridge Organic Wines
Claim type: leadership / CEO
Freshness: current preferred

No real searched names may be hardcoded in code, tests, fixtures, mocks, corpora, or proof hooks.

3.4 PH1.SEARCH or Current Search Planning Owner

Search planning must be structured, not loose strings.

It should produce:

original query
requested entity
normalized entity
claim target
role target
query variants
official-source targets
provider strategy
max provider calls
max pages to fetch
freshness tier
evidence requirements
visual/image need
language requirement

The requested entity must remain a hard constraint through the entire path unless the user explicitly broadens the question.

3.5 Provider Governance

Provider Governance owns:

provider/model allowlist
provider route selection
budget/call caps
provider-off proof
fake-provider proof
network dispatch counters
data-egress/privacy
model evidence
fallback policy
no startup probes
no hidden calls

Quality-first does not mean ungoverned provider chaos.

3.6 PH1.E — Search, Evidence, Source, Tool Owner

PH1.E owns:

provider execution
source candidates
safe page read where allowed
evidence extraction
source ranking
wrong-source rejection
claim verification
source chip creation
image metadata acceptance
search trace

PH1.E must not approve business actions.

PH1.E must not mutate protected data.

PH1.E must not fabricate evidence.

3.7 Page Read / Evidence Extraction Layer

Search snippets are not enough for high-quality answers.

Where safe and justified, Selene should read the source page.

This layer handles:

safe URL fetch
private/internal URL blocking
redirect limits
timeout limits
page size limits
content-type limits
script/style stripping
text extraction
evidence chunk extraction
quote/excerpt limits

External page text is evidence, not instruction.

3.8 PH1.WRITE — Final Human Presentation

PH1.WRITE receives structured answer material and produces:

direct answer
short explanation
same-language response
source chip display text
source card labels
image card captions
follow-up suggestions where useful
tts_text

PH1.WRITE must not:

invent facts
invent sources
hide uncertainty
upgrade weak evidence
override source verification
mix rejected sources into the normal answer
read debug/source metadata in TTS
bypass protected fail-closed law

3.9 Adapter

Adapter transports final response packets.

It must keep separate:

response_text / display_text
tts_text
source_chips
source_cards
image_cards
trace metadata

Adapter must not rewrite, rerank, summarize, or choose sources.

3.10 Desktop / iPhone

Clients render:

clean answer text
small clickable source chips
approved image cards
approved source cards
optional trace/debug only where authorized

Clients must not become search brain, writing brain, source ranker, or provider router.

3.11 TTS

TTS speaks only approved final tts_text.

TTS must not read:

source dump
full source list by default
debug trace
provider metadata
raw JSON
rejected-source reasons
long URLs

4. Required Packets

Selene needs strict packets, not loose strings.

Codex must reuse exact repo names where they exist and map architecture names to repo-equivalent packets where they do not.

4.1 SearchIntentPacket

user_text
language
search_needed
public_or_protected_lane
freshness_required
requested_entity
claim_type
confidence
visual_result_useful
clarification_needed

4.2 SearchPlanPacket

original_query
requested_entity
normalized_entity
claim_target
role_target
query_variants
official_source_targets
provider_strategy
max_provider_calls
max_pages_to_fetch
budget_estimate
freshness_tier
evidence_requirements
visual_requirements

4.3 ProviderSearchDecisionPacket

allowed
provider_or_tool
provider_tier
model_or_capability
max_calls
estimated_cost
quality_reason
budget_status
request_id
turn_id
provider_off_status
fake_provider_status

4.4 SourceCandidatePacket

source_id
title
domain
url
snippet
provider
source_type
trust_score
entity_match_score
freshness_score
weak_source_flags
visual_metadata_available

4.5 EvidencePacket

source_id
extracted_text_or_hash
evidence_excerpt
claim_supported
claim_type
confidence
retrieved_at
page_read_used

4.6 SourceDecisionPacket

source_id
accepted_or_rejected
reason
entity_match
claim_support
trust_score
freshness_score
contradiction_status

4.7 ClaimVerificationPacket

claim
supported
best_supported_answer
supporting_sources
contradicting_sources
confidence
uncertainty_reason
closest_reliable_result

4.8 SourceChipPacket

source_id
label
domain
safe_click_url
source_type
accepted = true
claim_refs
optional_icon_key

Source chips must be derived from accepted sources only.

4.9 SearchImagePacket

image_id
safe_image_url_or_approved_asset_path
thumbnail_url_if_approved
source_page_url
caption
source_label
query_relevance_score
display_allowed
image_type
provenance

Image cards are optional but expected for entity/company/person/product-style searches where relevant images are available and approved.

4.10 AnswerPacket

answer_class
direct_answer
best_available_answer
uncertainty
accepted_sources
rejected_source_summary_for_trace
claim_verification
source_chips
optional_image_cards
follow_up_suggestions
no_raw_provider_data

4.11 PresentationPacket

display_text
source_chips
source_cards
image_cards
tts_text
trace_id
language
metadata_safe_for_user

5. Best Available Answer Rules

Selene must always produce the best helpful answer it lawfully can.

5.1 If exact answer is supported

Answer directly.

Example:

Alex Stone is listed as CEO of Moonridge Organic Wines.
[Official Site] [Company Profile]

5.2 If exact answer is not supported but a close reliable result exists

Answer with the best supported narrower claim.

Example:

I found a reliable listing for Alex Stone as Managing Director, but not a verified CEO listing. Best available: Alex Stone appears to be the senior listed leader in the sources I found.
[Official Site]

This is not a refusal. It is a useful best-available answer.

5.3 If sources conflict

Give the safest synthesis and show the sources.

Example:

The sources disagree. The company site lists Alex Stone as Managing Director, while an older directory lists Pat Lee as CEO. I would treat the company site as stronger and newer unless you want me to dig deeper.
[Company Site] [Directory]

5.4 If the query is ambiguous

Ask one useful clarification only when needed.

But if a likely interpretation exists, give a provisional best-available answer and explain the assumption.

5.5 If no useful public evidence exists

Do not end with a cold failure.

Example:

I did not find a reliable public source for the exact claim. The best next step is to check the company’s official site, filings, or LinkedIn page. I found these closest sources:
[Official Site] [Registry]

Selene still helps.

6. Accuracy and Evidence Rules

Selene must not answer from weak evidence as if it is verified.

A source is not accepted just because it appears first.

Selene must check:

Is this source about the requested entity?
Does the title/domain/snippet/page match the entity?
Is this source official, primary, reputable, community, weak, spam, or stale?
Does this source actually support the claim?
Is the source current enough?
Are there contradictions?
Is the source only mentioning the entity without proving the claim?

Important rule:

Source mentions entity ≠ source proves claim.

Selene may answer with best-available uncertainty, but must not upgrade uncertainty into fake certainty.

7. Wrong-Source Rejection

Selene must reject or downgrade:

wrong entity
partial name overlap only
similar but different company/person/product
SEO pages
scraped profile pages
ranking spam
stale pages for current claims
sources that only mention the entity
sources that do not support the requested claim
sources with unresolved contradiction

The requested entity must remain a hard constraint through:

SemanticInterpreterProvider / GPT-5.5
→ PH1.X
→ PH1.N / entity extraction owner
→ PH1.SEARCH / search planning owner
→ PH1.E
→ PH1.WRITE

This prevents:

synthetic company query
→ drift to unrelated real source
→ answer with wrong source

That failure must become impossible after the first search repair stage.

8. Source Display Rules

Every websearch answer must show where the information came from.

User-facing search output must include:

direct answer first
visible accepted source chips
short source labels
safe clickable URLs hidden behind chips
no raw provider metadata
no unix_ms
no debug dump
no rejected sources in normal answer

Rejected sources remain in trace/debug only.

Bad output:

I found a web result...
Sources:
1. raw provider dump
2. weak source
3. unrelated source

Good output:

Best available: Alex Stone is listed as Managing Director, but I did not find a verified CEO listing in accepted sources.
[Official Site] [Company Registry]

9. Small Clickable Source Chips / Source Pills

Source chips must be:

small
clean
clickable
compact
elegant
easy to scan
same visual standard across Desktop/iPhone

Chip label examples:

Official Site
Company Page
Regulator
News Source
Documentation
LinkedIn
Registry
Research Paper
Community Discussion

Each chip must:

come from an accepted source
open the safe source-page URL
avoid raw tracking/debug metadata
avoid long visible URLs
avoid raw provider payload
map to supported claim refs where possible

Source chips may appear:

inline beside a key fact
under the direct answer
in a compact source row beneath the answer block

This is Selene’s standard source presentation style.

10. Rich Search Result Presentation

For public entity searches, Selene should not return plain text only when richer presentation is available.

Entity types include:

company
person
brand
product
organization
place
restaurant / hotel / venue
public project
software/library/API

Target layout:

header/title
relevant images/photos when approved
direct answer
short facts
source chips
optional follow-up suggestions

Example:

User: Who is the CEO of Moonridge Organic Wines?

Selene response:
[company/logo/photo if approved]
Best available: The strongest source I found lists Alex Stone as Managing Director. I did not find a verified CEO title in the accepted sources.

Key facts:
- Entity: Moonridge Organic Wines
- Role found: Managing Director
- CEO title: not confirmed in accepted sources

[Official Site] [Registry]

Want me to check company filings or LinkedIn next?

11. Query-Relevant Photos / Visual Cards

For company/person/brand/product/place searches, Selene should display relevant images when approved.

Target visual behavior:

1 to 3 relevant images
image strip/cards near the top of the response
exact-query relevance
not random stock images
not unrelated images
not raw image dumps

Valid image types:

company logo
key person photo
office / building / winery / storefront
product image
official visual asset
map/place image where allowed

Rules:

images must be relevant to the exact query
images must come from an approved source path
images must not be fabricated as real source images
unsafe/unverified image fetching must not be used
image rights/provider/display policy must be respected
if image path is not approved, degrade to text + source chips

Minimum:

text answer + source chips

Target:

text answer + source chips + relevant approved images

12. Accepted Source UX Standard

Selene must separate:

answer text
source chips
source cards
image cards
trace/debug

Correct output model:

response_text = clean answer only
source_chips = small clickable accepted-source chips
source_cards = optional larger source preview rows/cards
image_cards = optional approved relevant visual assets
trace = accepted/rejected source reasons and internal proof
tts_text = clean spoken answer only

Selene must not return:

I found a web result...
giant Sources dump
raw provider JSON
raw URL dump
rejected sources mixed into user answer
internal metadata in normal output

Selene should return:

direct answer
concise facts
small clean source chips
optional image strip/cards
optional follow-up suggestions
optional expandable deeper explanation

13. Visual Search Answer Templates

Selene should support structured answer templates.

13.1 Company / Person / Entity Lookup

Header/title
Image strip/cards if approved
Direct best-available answer
Key facts
Source chips
Optional follow-up suggestions

13.2 Comparison

Direct comparison summary
Side-by-side key facts
Source chips per claim block

13.3 Ranked List / Best Options

Direct recommendation summary
Short ranked bullets
Source chips
Caveats / criteria

13.4 News / Current Events

Headline summary
Short bullet recap
Freshness marker where useful
Source chips

13.5 Research Answer

Direct answer first
Short supporting explanation
Accepted source chips
Optional deeper evidence section

13.6 Time / Weather / Public Tool Answer

For time/weather-style public factual answers, the result should be:

short by default
natural language
specific to requested place/time range
format matched to user request
TTS-safe
source/tool provenance available in trace

Examples:

What is the weather for the next 5 days?
→ 5-day concise forecast, not a one-day dump.

What was the weather last week?
→ historical range if provider supports it, or best-available source-backed answer with limits stated naturally.

Tool facts come from PH1.E/tool/provider.

Human presentation comes through PH1.WRITE / Quick Assist / Selene style where allowed.

14. Provider Strategy: Best-Fit, Not Cheap-First

Remove cheap-first as architecture.

Use best-fit provider strategy.

Provider choice must optimize for:

answer quality
freshness
source reliability
entity precision
claim verification needs
visual availability
latency
cost governance
provider policy

Cost remains governed, but Selene must not choose a low-quality source path that fails the user just because it is cheap.

Provider lanes:

Lane 0 — No Search

For stable explanations, writing, transformations, and summaries where external freshness is not needed.

Lane 1 — Cache / Recent Accepted Evidence

Use recent verified results where policy allows.

Lane 2 — Standard High-Quality Search

Default for normal public source-backed search.

Lane 3 — Current / News / Freshness Search

For latest/current/news/time-sensitive questions.

Lane 4 — Entity / Company / Person Precision Search

For entity-specific questions requiring wrong-source rejection, official sources, page reads, and claim verification.

Lane 5 — Visual Search / Image Metadata Path

For searches where images improve the answer.

Lane 6 — Deep Research

For explicit or approved high-effort research with multiple sources, page reads, contradiction detection, and possibly user-visible cost/effort notice.

Premium or paid providers may be used when justified by quality, user request, freshness, or source-verification need, but always through Provider Governance.

15. Provider Governance and Cost Control

Quality-first does not mean cost-blind.

Rules:

no startup provider probes
no provider call without governance preflight
no live paid provider in normal tests
no hidden provider fallback
no provider fanout by default
provider-off = zero attempts and zero dispatches
call caps per lane
budget evidence before network dispatch
safe degrade when provider unavailable

Budget exhausted should not produce a cold failure.

It should produce a useful degraded answer:

I can give a general answer now, but live source-backed search is unavailable in this mode.

or if some accepted cached/source evidence exists:

Best available from recent accepted sources...

16. Controlled Brave / Premium Provider Re-Enable

Any premium provider such as Brave must remain governed.

Premium providers may be enabled only after:

global provider kill switch proven
provider call counter proven
provider budget gate proven
no startup provider probes proven
no paid provider in normal tests
provider-off = zero call attempt proof
per-turn call caps proven
model/provider policy approved

Controlled capped mode requires:

one approved key
low-volume test
explicit cap
no background calls
no startup probes
no deep research fanout by default
logging before each call

Proof requirements:

one query = expected call count
no hidden extra calls
no fallback surprise calls
no startup calls
no provider leak while off
source chips display correctly
image behavior correct where approved
clean direct answer remains clean

17. Enterprise Trace and Audit

Every websearch answer should produce trace/audit.

Trace must record:

turn_id
request_id
captured transcript hash
semantic proposal refs
PH1.X directive refs
normalized intent
requested entity
search plan
expanded queries
provider selected
provider call count
budget approval
source candidates
accepted sources
rejected sources
rejection reasons
evidence extracted
claim verification result
source chips delivered
image cards delivered where approved
final answer class
display_text hash
tts_text hash

Trace is for audit and debugging.

Normal users should not see trace unless authorized or requested in an approved mode.

18. Protected Execution Boundary

Public websearch remains public read-only answer work.

Protected business execution remains authority + simulation gated.

Search results must never become protected execution authority.

Examples:

Search payroll rules
→ allowed public search where policy permits

Approve payroll
→ protected fail-closed unless authority + simulation pass

Search rules and approve payroll
→ answer the search part
→ protected part fails closed unless authorized

PH1.WRITE must separate public answer from protected denial.

19. No Hardcoded Search Names

No real customer, company, person, product, supplier, competitor, or searched name may be hardcoded in production code, tests, fixtures, mocks, corpora, sample data, or proof hooks.

Allowed only in:

docs
ledger history
user-provided reports
manual proof notes

Tests must use synthetic fake entities.

If Selene fails on a real entity, Codex must fix the generic capability:

spelling
phonetic matching
entity disambiguation
query planning
source ranking
page reading
claim verification
answer formatting

Do not patch the real searched name into code.

20. Build Order

Do not build everything in one pass.

Stage 0 — PH1.E Search Intelligence Lane Activation Pack

Before implementation, Codex must map repo truth:

current PH1.E search/source packets
current PH1.N/entity extraction if any
current PH1.SEARCH planning owner if any
current provider governance
current source chips
current image cards
current page fetch/read path
current claim verification
current adapter response separation
current Desktop/iPhone rendering
current tests/evals
current old search paths

Stage 1 — Search Intent + Best-Fit Planning Baseline

Build:

SearchIntentPacket / repo equivalent
SearchPlanPacket / repo equivalent
semantic proposal → PH1.X → PH1.E search directive
entity hard-constraint preservation
no-search vs search decision

Proof:

no phrase patches
no real searched-name hardcoding
provider-off zero attempts

Stage 2 — Provider Governance + Best-Fit Provider Route

Build:

provider preflight
call caps
budget evidence
provider-off zero attempts
fake-provider proof
model/provider policy enforcement

Remove cheap-first language.

Use best-fit provider strategy.

Stage 3 — Wrong-Source Rejection + Entity Constraint Repair

Build:

entity match scoring
wrong-source rejection
partial-name overlap rejection
claim support requirement
accepted/rejected source trace

Fix known failure pattern:

query drifts to unrelated source
→ wrong source accepted
→ bad answer/source dump

Stage 4 — Page Read + Evidence Extraction

Build:

safe URL fetch
page read
content extraction
evidence chunks
private/internal URL blocking
limits/timeouts

Stage 5 — Claim Verification + Best Available Answer

Build:

claim-to-source mapping
contradiction handling
best-supported answer selection
closest reliable result behavior
unsupported claim downgrade

Selene should return best available answer, not dead-end refusal.

Stage 6 — PH1.WRITE Search Presentation + Source Chips

Build:

direct short answer
small clickable source chips
same-language response
TTS-safe final text
no raw provider/debug metadata
no rejected sources in normal answer

Stage 7 — Rich Entity Layout + Image Cards

Build:

entity layout
1–3 approved relevant images
image-card safety
source-linked visuals
no fake image attribution

Stage 8 — Time / Weather / Public Tool Presentation Upgrade

Build:

range-aware forecast presentation
historical weather/time handling where provider supports it
short/default answer style
user-requested format adaptation
PH1.WRITE/Selene/Quick Assist presentation

Stage 9 — Premium Provider Controlled Re-Enable

Build:

controlled Brave/premium provider mode if approved
low-volume billing proof
hidden-call proof
call count proof
source chip proof

Stage 10 — Advanced Research / Better-than-ChatGPT Upgrade Layer

Build:

benchmark harness
hard question corpus
source agreement scoring
freshness scoring
multi-provider corroboration where approved
deep research reports
voice-first search tests
visual entity answers

21. Success Standard

Selene Search Intelligence is not finished until it can do this:

User asks current/entity/factual question
→ Selene understands whether search is needed
→ Selene plans query with hard entity constraints
→ Selene uses best-fit provider strategy through governance
→ Selene reads pages where needed and safe
→ Selene rejects wrong or weak sources
→ Selene verifies claims against evidence
→ Selene gives best available helpful answer
→ Selene answers directly and concisely
→ Selene shows small clickable source chips
→ Selene shows relevant images where approved
→ Selene speaks only final answer text
→ Selene logs trace, cost, and evidence
→ Selene preserves protected fail-closed law

22. Final UX Target

Selene search answers should feel:

human
clear
fast
visual
source-backed
trustworthy
beautiful
easy to scan
short by default
expandable on request
same or better than ChatGPT-level presentation

Target entity/company result:

header
+ relevant approved images
+ direct best-available answer
+ short verified facts
+ small clickable source chips
+ optional follow-up suggestions

23. Final Design Summary

Search quality comes from:

OpenAI-assisted semantic understanding
best-fit provider strategy
entity hard constraints
source ranking
page read
evidence extraction
wrong-source rejection
claim verification
best available answer synthesis

Human UX comes from:

PH1.WRITE presentation
Selene / Quick Assist wording where appropriate
source chips
image cards
same-language response
TTS-safe final text
short-by-default style
natural uncertainty wording

Enterprise quality comes from:

provider governance
model policy
budget/call caps
trace
audit
accepted/rejected source proof
claim-to-source mapping
protected fail-closed law
no hardcoded search-name patches
Desktop/iPhone render-only
Adapter transport-only

The correct build path is:

Search Intelligence Activation Pack
→ Search intent + best-fit planning
→ Provider governance route
→ wrong-source rejection
→ page read/evidence
→ claim verification + best available answer
→ PH1.WRITE/source chips
→ rich entity layout + image cards
→ time/weather/public tool presentation upgrade
→ controlled premium provider re-enable
→ advanced research and evaluation

This is the clean path to making Selene’s websearch human, accurate, visual, source-backed, OpenAI-assisted, enterprise-governed, and better than a shallow search wrapper.
