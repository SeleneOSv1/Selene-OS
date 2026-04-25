# H360 - PH1.E Global Deterministic Time Place Normalization And TZDB Rendering Build Plan

## Purpose

H360 is the narrow build-plan run after H359.

H359 generalized deterministic time rendering enough to prove clean live desktop answers for New York, Japan, and Sydney. That is useful, but it is not yet the final global deterministic time architecture. Selene must be able to answer supported country, city, and timezone time questions globally without using web search snippets, raw provider dumps, fake local tables, or a provider-written final sentence.

H360 defines the next lawful seam: global deterministic place normalization plus canonical timezone resolution plus clean PH1.X final-answer rendering for time only.

This is not shell redesign, PH1.D redesign, weather work, generic backend expansion, or protected-action law widening.

## Authority

- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1e.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1n.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md`

## Repo Truth Proof

- The highest committed H-series plan before this document is H359, so the next free implementation id is H360.
- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1e.rs` already routes `ToolName::Time` through deterministic time handling.
- H359 added system-tzdb-backed time handling and clean local-time rendering for New York, Japan, and Sydney.
- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1e.rs` now reads local IANA tzdb metadata where available, but it does not yet prove provider-neutral global geocoding for arbitrary supported places.
- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1n.rs` already classifies time questions as deterministic `TimeQuery`.
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs` already renders `ToolResult::Time` through a clean final-answer path.
- `/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs` already has adapter-level deterministic time acceptance tests for known locations.
- No repo truth justifies using web search as the default time provider.
- No repo truth justifies protected-action law widening for low-risk public deterministic time answers.

## CURRENT

- Some deterministic time paths already exist.
- New York, Japan, and Sydney have clean deterministic time proof from H359.
- Some supported locations can still fall through to raw ISO or incorrect formatting if they are not normalized through the known deterministic place/timezone path.
- Current handling is not yet globally normalized across all supported countries, cities, and timezone names.
- Current proof is not enough for "any country" accuracy.
- Current repo truth does not prove a lawful global geocoder/timezone-provider configuration is present for every place query.

## TARGET

- Deterministic time works globally for supported places.
- Country, city, and timezone queries resolve lawfully through deterministic place normalization.
- UTC comes from an NTP-synced system clock, not web search.
- Timezone and DST rules come from IANA tzdb.
- Place resolution uses a provider-neutral geocoder/normalizer, not web search snippets.
- Primary place-normalization options are Nominatim and GeoNames.
- Timezone lookup options are TimeZoneDB or GeoNames timezone lookup by coordinates.
- Selene computes local time from:
- the resolved place,
- the canonical timezone,
- the current UTC/system clock,
- and IANA tzdb DST rules.
- Single-timezone countries answer directly.
- Multi-timezone countries clarify instead of guessing silently.
- Ambiguous place names ask a clean clarification.
- Final user answers are always clean and human-readable.
- No raw ISO, raw provider payload, raw source dump, or web-search snippet appears in the final deterministic time answer.
- No protected-action law is widened.

## GAP

This is not shell redesign, sidebar redesign, composer redesign, PH1.D redesign, weather work, generic backend expansion, identity/access/simulation weakening, or protected-action law widening.

The exact gap is global place normalization plus timezone resolution plus clean deterministic final-answer rendering for time.

## In Scope

- Add or refine deterministic global place normalization for time queries.
- Resolve supported cities, countries, and explicit IANA timezone names to canonical timezone IDs.
- Use provider-neutral place normalization first, with Nominatim or GeoNames as approved primary options.
- Use TimeZoneDB or GeoNames timezone lookup by coordinates if a location resolves by geocoder coordinates.
- Use the NTP-synced system clock as the UTC source.
- Use IANA tzdb for timezone and DST conversion.
- Compute the local time in PH1.E from canonical timezone plus current UTC.
- Render the final answer in PH1.X as a clean human sentence.
- Preserve PH1.N deterministic `TimeQuery` classification.
- Preserve PH1.X dispatch to the deterministic time tool lane.
- Add tests for countries, cities, and timezone names across multiple regions.
- Add tests for single-timezone countries answering directly.
- Add tests for multi-timezone countries clarifying instead of guessing.
- Add tests proving no deterministic time path uses web search unless the user explicitly asks to search the web.

## Out of Scope

- No shell redesign.
- No sidebar redesign.
- No composer redesign.
- No PH1.D provider redesign.
- No weather work in this run.
- No backend route expansion unless repo truth proves unavoidable.
- No protected-action law widening.
- No identity/access/simulation weakening.
- No fake local answer lane.
- No fake static global place table.
- No provider-written final time sentence.
- No web-search fallback for deterministic time unless the user explicitly asks to search the web.
- No iPhone implementation work.
- No DB wiring spec rewrite.

## Files Allowed To Change

- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1e.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1n.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs`

If and only if implementation succeeds and repo law requires landed-truth updates:

- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md`

## Files Not Allowed To Change

- Desktop shell files.
- RuntimeBridge files.
- PH1.D provider or contract files.
- Backend route expansion files, unless repo truth proves unavoidable; if so, stop and report first.
- DB wiring specs.
- iPhone source files.
- Unrelated runtime law files.
- Any file outside the allowed scope unless repo truth proves unavoidable; if so, stop and report first.

## Acceptance Standard

- `what is the time in New York` returns a clean deterministic final answer.
- `what is the time in Italy` returns a clean deterministic final answer if Italy resolves as a single-timezone country.
- `what is the time in Japan` returns a clean deterministic final answer.
- `what is the time in Sydney` returns a clean deterministic final answer.
- Additional supported countries, cities, and explicit timezone queries resolve cleanly across multiple regions.
- Multi-timezone countries clarify instead of guessing silently.
- Ambiguous place names clarify cleanly instead of leaking raw data.
- Unsupported places fail closed cleanly.
- Final user answers never expose raw ISO strings, raw provider payloads, raw source dumps, HTML entities, or web-search snippets.
- Web search is not used for deterministic time unless the user explicitly asks to search the web.
- No protected-action, identity/access, or simulation law is widened.

## Exact Test Commands

```sh
git -C /Users/selene/Documents/Selene-OS status --short
git -C /Users/selene/Documents/Selene-OS branch --show-current
git -C /Users/selene/Documents/Selene-OS rev-parse HEAD
cargo test -p selene_engines at_n_39_time_in_japan_and_sydney_are_deterministic_time_queries
cargo test -p selene_engines at_e_time_query_returns_current_new_york_time
cargo test -p selene_engines at_e_time_query_returns_current_japan_time
cargo test -p selene_engines at_e_time_query_returns_current_sydney_time
cargo test -p selene_os at_x_tool_ok_time_japan_renders_clean_final_answer_without_raw_iso
cargo test -p selene_os at_x_tool_ok_time_sydney_renders_clean_final_answer_without_raw_iso
cargo test -p selene_adapter at_adapter_4
```

The implementation run must add exact new tests for:

- Italy or another single-timezone country returning a clean answer.
- At least one explicit IANA timezone query returning a clean answer.
- At least one city resolved through the global place-normalization path.
- At least one multi-timezone country clarifying instead of guessing.
- At least one ambiguous place clarifying cleanly.
- At least one unsupported place failing closed cleanly.
- Proof that deterministic time does not use web search unless explicitly requested.

## Stop Conditions

- Stop if the next free id is not H360.
- Stop if the tree is dirty before implementation.
- Stop if global time accuracy would require protected-action law widening.
- Stop if provider-neutral place normalization cannot be done inside the approved scope.
- Stop if required geocoder or timezone-provider configuration is missing; report `H360_PLACE_PROVIDER_CONFIG_BLOCKER` instead of inventing a fake local table.
- Stop if timezone resolution requires files outside the approved scope.
- Stop if current repo truth disproves deterministic country/city/timezone handling in this family.
- Stop if the only passing path would be web-search fallback instead of deterministic time handling.
- Stop if UTC would have to come from web search instead of the NTP-synced system clock.
- Stop if IANA tzdb is unavailable and no lawful repo-supported tzdb source exists.
- Stop if multi-timezone country handling would require guessing silently.
- Stop if final-answer rendering would expose raw ISO strings, raw provider payloads, source dumps, or web snippets.

## Ledger Update Rule

Update `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md` and `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md` only after successful implementation and acceptance proof, if repo law requires landed-truth updates.
