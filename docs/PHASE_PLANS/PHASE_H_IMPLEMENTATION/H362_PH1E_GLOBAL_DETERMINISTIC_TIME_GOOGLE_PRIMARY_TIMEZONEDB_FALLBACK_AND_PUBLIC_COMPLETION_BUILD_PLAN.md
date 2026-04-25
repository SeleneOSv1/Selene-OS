# H362 PH1E Global Deterministic Time Google Primary TimeZoneDB Fallback And Public Completion Build Plan

## Purpose

H362 defines the next lawful deterministic-time seam after the bounded H358/H360 proof and the H361 weather planning work.

The goal is for Selene to answer time accurately for any lawfully resolvable place in the world without routing deterministic time through web search, PH1.D general answer generation, or protected identity gates.

Low-risk public time queries must not require Voice ID. If the place is clear and maps to one timezone, Selene answers directly. If the place is ambiguous or spans multiple timezones, Selene asks a short clarification instead of guessing.

This is not a shell redesign, sidebar redesign, composer redesign, PH1.D redesign, generic backend expansion, wake-law change, session-law change, or protected-action law widening.

## Authority

- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1e.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1n.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/app_ingress.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs`
- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`
- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/provider_secrets.rs`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md`

## Repo Truth Proof

- The next free H implementation id is H362.
- H358 proved a bounded deterministic New York happy path only.
- H360/H361 planning established that deterministic time and weather require global provider normalization and clean final rendering, but H362 is time-only.
- Current deterministic time succeeds for some supported places, including New York, Japan, Sydney, and Italy in existing repo/runtime proof.
- Current live/runtime proof shows Portugal and Germany can still fail through identity/gating even though they are low-risk public deterministic time queries.
- The remaining Portugal/Germany failure is not primarily a raw ISO, raw payload, or web-search dump problem. It is a deterministic public-completion and gating problem.
- Current provider secret ids already exist:
  - `google_time_zone_api_key`
  - `timezonedb_api_key`
- Existing deterministic tests prove some bounded time paths, but current proof is not enough for "any lawfully resolvable place in the world."
- No current repo truth justifies protected-action law widening for low-risk public deterministic time.

## CURRENT

- Deterministic time currently works for New York, Japan, Sydney, Italy, and some other supported places.
- Portugal and Germany can still fail through identity/gating instead of returning a low-risk public deterministic time result.
- Current handling is not yet globally normalized across supported countries, regions, cities, and IANA timezone names.
- Current proof is not sufficient for global country/place correctness.
- Google Time Zone and TimeZoneDB vault secret ids already exist.
- No current proof justifies protected-action, identity, access, session, wake, or simulation law widening.

## TARGET

- Supported low-risk public deterministic time queries complete without Voice ID.
- UTC comes from an NTP-synced system clock as the base time truth.
- IANA tzdb provides timezone and DST rules truth.
- User place text is normalized into canonical place truth before timezone lookup.
- Google Time Zone API is the primary timezone-resolution source.
- TimeZoneDB is the secondary fallback only when Google is unavailable, fails, times out, or returns no lawful result.
- Selene computes local time deterministically from:
  - current UTC
  - canonical place truth
  - canonical timezone truth
- Clear single-timezone places answer directly.
- Ambiguous or multi-timezone places ask a short clarification instead of guessing.
- Final user answer is clean and concise, for example:
  - `It's 2:50 AM in Rome.`
- Final user answer never exposes:
  - raw ISO strings
  - raw provider payloads
  - raw source dumps
  - web-search snippets
- Existing lawful TTS may speak the completed answer when enabled and lawful.
- No fake local answer lane is invented.

## GAP

This is not shell redesign, sidebar redesign, composer redesign, PH1.D redesign, generic backend expansion, or protected-action law widening.

The exact gap is global place normalization plus timezone resolution plus deterministic local-time computation plus public deterministic completion plus identity-gate bypass for low-risk time queries.

## In Scope

- Global deterministic place normalization for time queries.
- Country, region/state, city, and IANA timezone input handling where lawfully resolvable.
- Google Time Zone API primary timezone lookup.
- TimeZoneDB secondary fallback timezone lookup.
- Current UTC use from an NTP-synced system clock.
- IANA tzdb use for timezone and DST rules.
- PH1.N preservation of time questions as `TimeQuery`.
- PH1.X dispatch of `TimeQuery` to the deterministic time lane.
- Clean final-answer rendering for deterministic time.
- Clarification for ambiguous or multi-timezone places.
- Low-risk public deterministic time completion without Voice ID.
- Preservation of protected memory, personalization, permissions, and protected actions behind current identity/access law.
- Returning completed time results through app ingress, adapter response, desktop runtime bridge, and visible main pane.
- Existing lawful TTS playback for completed answers.

## Out of Scope

- Shell redesign.
- Sidebar redesign.
- Composer redesign.
- PH1.D provider redesign.
- PH1.D provider calls for deterministic time.
- Weather work.
- Generic backend expansion.
- Fake adapter-local or desktop-local answer fabrication.
- Web-search fallback for deterministic time.
- New secret ids.
- Protected-action law widening.
- Identity/access/simulation weakening.
- Wake-law changes.
- Session-law changes.
- Hidden/background wake work.
- False "works" claims without live desktop or exact repo-supported runtime proof.

## Files Allowed To Change

- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1e.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1n.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/app_ingress.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs`
- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`
- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`

Only if repo truth proves unavoidable, stop and report first before editing exact realtime/provider configuration files already used by this seam.

If and only if implementation succeeds and repo law requires landed-truth updates:

- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md`

## Files Not Allowed To Change

- PH1.D contract files.
- Backend route expansion files.
- DB wiring specs.
- iPhone source files.
- Unrelated desktop source files.
- Unrelated runtime law files.
- Any new provider secret-id definition file unless repo truth proves the current secret ids are wrong, in which case stop and report first.

## Acceptance Standard

H362 is accepted only when all of the following are true:

- `what is the time in New York` returns a clean deterministic final answer.
- `what is the time in Germany` returns a clean deterministic final answer.
- `what is the time in Sydney` returns a clean deterministic final answer.
- `what is the time in Lisbon` returns a clean deterministic final answer.
- `what is the time in Tokyo` returns a clean deterministic final answer.
- `what is the time in Japan` returns a clean deterministic final answer.
- `what is the time in America/New_York` returns a clean deterministic final answer.
- `what is the time in Portugal` clarifies cleanly instead of guessing, for example asking whether the user means mainland Portugal/Lisbon or the Azores.
- `what is the time in United States` clarifies cleanly instead of guessing across multiple timezones.
- Ambiguous city queries such as `what is the time in Springfield` clarify cleanly instead of guessing.
- Portugal and Germany no longer fail through Voice ID, strong-identity, or protected-action gating.
- Final answers do not expose raw ISO strings, raw provider payloads, raw source dumps, web-search snippets, or `Retrieved at (unix_ms)`.
- Web search is not used for deterministic time unless the user explicitly asks to search the web.
- Google Time Zone is used as primary.
- TimeZoneDB is used only as secondary fallback.
- Protected actions, memory, personalization, permissions, and identity-scoped behavior remain fail-closed under current law.

## Exact Test Commands

Preflight:

```bash
git -C /Users/selene/Documents/Selene-OS status --short
git -C /Users/selene/Documents/Selene-OS branch --show-current
git -C /Users/selene/Documents/Selene-OS rev-parse HEAD
```

Static and build checks:

```bash
cargo check -p selene_engines -p selene_os -p selene_adapter
cargo test -p selene_kernel_contracts provider_secret_ids_are_roundtrippable
```

Existing deterministic time regression checks:

```bash
cargo test -p selene_adapter at_adapter_40_time_in_new_york_returns_clean_current_time_answer
cargo test -p selene_adapter at_adapter_41_time_in_japan_returns_clean_current_time_answer
cargo test -p selene_adapter at_adapter_42_time_in_sydney_returns_clean_current_time_answer
```

H362 implementation must add or identify exact runnable tests proving:

```bash
cargo test -p selene_adapter h362_time_in_germany_returns_clean_public_answer
cargo test -p selene_adapter h362_time_in_lisbon_returns_clean_public_answer
cargo test -p selene_adapter h362_time_in_portugal_clarifies_mainland_or_azores
cargo test -p selene_adapter h362_time_in_united_states_clarifies_multi_timezone
cargo test -p selene_adapter h362_time_in_iana_zone_returns_clean_public_answer
cargo test -p selene_adapter h362_time_google_primary_timezonedb_fallback
cargo test -p selene_adapter h362_low_risk_time_does_not_require_voice_id
cargo test -p selene_adapter h362_protected_actions_remain_identity_gated
```

The implementation run must not cite PH1.N tests as passing unless exact runnable test names are present in repo truth.

## Required Live Desktop Proof

The later implementation run must launch the actual macOS desktop app or the exact repo-supported desktop runtime path and ask:

- `what is the time in New York`
- `what is the time in Germany`
- `what is the time in Sydney`
- `what is the time in Lisbon`
- `what is the time in Tokyo`
- `what is the time in Japan`
- `what is the time in America/New_York`
- `what is the time in Portugal`
- `what is the time in United States`

The proof must show:

- Direct answers appear for clear/single-timezone places.
- Clarification appears for ambiguous or multi-timezone places.
- Portugal and Germany no longer fail through identity/gating.
- No raw ISO, raw provider payload, raw source dump, web-search snippet, or `Retrieved at (unix_ms)` appears in the final answer.
- Desktop bridge or exact runtime logs show completion, not a dropped/withheld/identity-gated request.
- Existing lawful TTS can speak the completed answer when enabled and lawful.
- Screenshot proof of the visible main pane after completion is included when the app path is used.

If the actual app cannot be launched, the implementation run may use the exact repo-supported desktop runtime path only if it proves the same app ingress, adapter response, and desktop bridge completion path. It must stop and report instead of substituting unrelated unit tests for live acceptance.

## Stop Conditions

- Stop if truthful current time requires provider use outside NTP-synced UTC, IANA tzdb, Google Time Zone primary, and TimeZoneDB secondary.
- Stop if low-risk public deterministic time still requires Voice ID or strong identity verification.
- Stop if lawful place normalization/geocoding requires files outside the approved scope.
- Stop if timezone resolution requires backend route expansion.
- Stop if desktop completion requires protected-action authority.
- Stop if the live desktop app or exact repo-supported desktop runtime path cannot be launched/tested.
- Stop if the result can only pass by fabricating a desktop-local or adapter-local answer.
- Stop if the only passing path would be web-search fallback instead of deterministic time handling.
- Stop if TTS playback requires new runtime authority instead of existing carriers.
- Stop if new secret ids would be required for this seam.
- Stop if Google Time Zone or TimeZoneDB vault secrets are unavailable.
- Stop if ambiguous/multi-timezone handling cannot clarify without guessing.

## Ledger Update Rule

If H362 implementation succeeds, update landed-truth docs only if repo law requires it:

- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md`

If H362 stops, fails acceptance, or requires scope expansion, do not update landed-truth docs as complete. Report the blocker and leave the completion ledger untouched.
