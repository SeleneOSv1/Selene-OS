# STAGE9_SEARCH_EVALUATION_AND_ADVANCED_RESEARCH_UPGRADE Certification Report

Date: 2026-05-02

Readiness class: READY_EXCEPT_REAL_VOICE_NOT_PROVEN

Live provider proof: NOT RUN

Real Desktop voice proof: NOT PROVEN

## Corpus Summary

Offline synthetic certification corpus:

| Case | Coverage |
| --- | --- |
| stage9_case_001_exact_entity_lookup | exact entity lookup |
| stage9_case_002_phonetic_entity_lookup | misspelled/phonetic entity lookup |
| stage9_case_003_overlap_trap | partial name overlap trap |
| stage9_case_004_wrong_source_drift | wrong-source drift trap |
| stage9_case_005_weak_source_rejection | weak SEO source trap |
| stage9_case_006_official_source_preference | official-source preference |
| stage9_case_007_role_ambiguity | leadership/role ambiguity |
| stage9_case_008_entity_only_insufficient | entity-only source insufficient |
| stage9_case_009_conflicting_sources | conflicting sources |
| stage9_case_010_stale_vs_fresh | stale source versus fresh source |
| stage9_case_011_numeric_contradiction | numeric contradiction |
| stage9_case_012_date_contradiction | date/event contradiction |
| stage9_case_013_current_news | current news/freshness question |
| stage9_case_014_cache_hit | cache hit question |
| stage9_case_015_cache_stale | cache stale question |
| stage9_case_016_cheap_provider_preferred | cheap provider preferred |
| stage9_case_017_premium_fallback_fake | premium fallback needed, fake provider only |
| stage9_case_018_deep_research_cap | Deep Research approval/cap required |
| stage9_case_019_source_chip | source chip display |
| stage9_case_020_image_allowed | image card allowed fixture |
| stage9_case_021_image_blocked | image card blocked fixture |
| stage9_case_022_tts_clean | TTS clean answer |
| stage9_case_023_same_language | same-language answer |
| stage9_case_024_protected_mixed | protected mixed prompt fail-closed |
| stage9_case_025_no_search | no-search prompt |
| stage9_case_026_unsupported_safe_degrade | unsupported claim safe-degrade |
| stage9_case_027_contradicted_safe_degrade | contradicted safe-degrade |
| stage9_case_028_page_read_beats_snippet | page-read evidence beats snippet |
| stage9_case_029_provider_off | provider-off safe response |
| stage9_case_030_more_detail | long answer requested after short answer |

Scorecard summary:

- total_cases: 30
- pass_count: 30
- fail_count: 0
- blocked_count: 0
- live_provider_call_attempt_count: 0
- live_provider_network_dispatch_count: 0
- url_fetch_count: 0
- image_fetch_count: 0
- fake_provider_call_count: deterministic offline only

## Provider-Off Proof

The Stage 9 offline certification report keeps live provider counters at zero. Runtime provider-off proof is a separate final gate and must also show:

- provider_call_attempt_count = 0
- provider_network_dispatch_count = 0
- url_fetch_count = 0
- image_fetch_count = 0

## Source Agreement Proof

Stage 9 adds `SourceAgreementPacket` / `Stage9SourceAgreementPacket` scoring. Source agreement supports claim verification; it does not replace claim verification. Strong official/primary fixture support can resolve conflicts; unresolved conflicts safe-degrade.

## Freshness Proof

Stage 9 adds `FreshnessScorePacket` / `Stage9FreshnessScorePacket`. Current claims require a freshness window. Stale or unknown-date risky evidence cannot produce confident current answers.

## Source Chips, Images, TTS

Offline certification verifies source-chip presence for source-backed cases, image cards only where fixture display is approved, image-blocked behavior where display is not approved, and clean `tts_text` without raw provider/debug/source metadata.

## Protected Fail-Closed

The committed protected mixed-command corpus case uses:

```text
Search the web and approve payroll for Fixture Employee Alpha.
```

The public search path remains separate and policy-bound. Payroll approval remains blocked without simulation and authority.

## Deep Research And Corroboration

Advanced Deep Research is packet-ready through `DeepResearchPlanPacket` and `DeepResearchReportPacket`, but default caps remain zero and approval is required. Multi-provider corroboration policy exists and remains OFF by default.

## Verification

- `git merge-base --is-ancestor 45b8979950bdb6e1ebc746b6d1fc38e1ba8fa766 HEAD` => pass
- `cargo check -p selene_os -p selene_adapter -p selene_engines` => pass with pre-existing adapter dead-code warnings
- `cargo test -p selene_engines stage9 -- --test-threads=1` => 6 passed
- `cargo test -p selene_os stage9 -- --test-threads=1` => 2 passed
- `cargo test -p selene_adapter stage9 -- --test-threads=1` => 2 passed
- `cargo test -p selene_engines search_certification -- --test-threads=1` => 6 passed
- `cargo test -p selene_os search_certification -- --test-threads=1` => 2 passed
- `cargo test -p selene_adapter search_certification -- --test-threads=1` => 2 passed
- `cargo test -p selene_adapter -- --test-threads=1` => lib 273 passed, 3 ignored, plus bins/integration passed
- `cargo test -p selene_os -- --test-threads=1` => lib 1375 passed plus bins/doc-tests passed
- `cargo test -p selene_engines -- --test-threads=1` => 637 passed, 12 ignored
- Runtime provider-off proof with `selene_adapter_http` on `127.0.0.1:18080` => adapter started, `/healthz` healthy, startup probe blocked before attempt/network, process-scoped `lsof` showed only the local listener
- Provider/path scan => reviewed; Stage 9 added no live direct provider path, hidden fallback, or default fanout
- Presentation leak scan => remaining hits are negative tests, contracts, or debug/trace-only guard strings
- Forbidden real-name scan => 0 matches outside docs/target; top-level `/tests` directory absent and reported as absent/skipped, not passed
- `git diff --check` => pass
- `xcodebuild -project /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop.xcodeproj -scheme SeleneMacDesktop -configuration Debug build` => BUILD SUCCEEDED
- Generated artifact cleanup => untracked `crates/selene_os/.runtime/` removed after `git ls-files` proved it was untracked

## Deferred Items

- live provider comparison: deferred
- live provider proof: not run
- real Desktop voice proof: not proven
- production billing monitoring dashboard: not built
- richer UI polish: deferred
- long-term durable cache: deferred unless already provided by Stage 8 surfaces
- additional live providers: not added
- advanced report UX: packet-ready, not a Desktop redesign
