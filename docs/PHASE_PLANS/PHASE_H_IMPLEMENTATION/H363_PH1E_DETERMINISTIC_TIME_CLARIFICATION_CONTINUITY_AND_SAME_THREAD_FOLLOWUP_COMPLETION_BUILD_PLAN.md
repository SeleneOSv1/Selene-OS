# H363 PH1E Deterministic Time Clarification Continuity And Same Thread Followup Completion Build Plan

## Purpose

H363 is the next narrow build authority after H362 for deterministic time only.

H362 improved global deterministic time resolution and proved clean low-risk public time completion for supported direct queries, but it did not finish multi-turn clarification continuity for ambiguous or incomplete time queries. The current product can ask a lawful time clarification, then lose the pending deterministic time context when the user replies with a place such as `Madrid` or `Lisbon`.

The H363 purpose is to make low-risk public deterministic time clarification behave like one continuous same-thread request globally: incomplete, ambiguous, or multi-timezone time questions ask one clean clarification, and the immediate lawful follow-up completes the same pending time request without Voice ID, protected-action widening, fake local answer behavior, PH1.D redesign, provider redesign, shell redesign, or backend route expansion.

H363 is not a Spain/Portugal-only correction. Spain and Portugal are examples of the generic clarification-continuity law for all ambiguous or multi-timezone places.

## Authority

- `/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H362_PH1E_GLOBAL_DETERMINISTIC_TIME_GOOGLE_PRIMARY_TIMEZONEDB_FALLBACK_AND_PUBLIC_COMPLETION_BUILD_PLAN.md`
- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1e.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1n.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/app_ingress.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs`
- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`
- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md`

## Repo Truth Proof

- The next free H implementation id is H363. Current repo plan ids end at H362.
- H362 is implemented and recorded in `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md` and `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md`.
- H362 preserved host/system UTC as current-time truth, IANA timezone identity as deterministic timezone/DST truth, Google Time Zone as primary provider-resolution source, and TimeZoneDB as secondary fallback.
- H362 made low-risk public deterministic time answers capable of completing through the adapter without Voice ID while protected actions remain identity-gated.
- In `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1e.rs`, incomplete normalized time location text can still resolve to `DefaultUtc`, so a query such as `what time is it in` can still return UTC/raw ISO-shaped time truth instead of a clean missing-place clarification.
- In `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1e.rs`, ambiguous time locations produce `ambiguous_time_location alternatives=...`, and `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs` turns that into a clean user-facing clarification.
- In `/Users/selene/Documents/Selene-OS/crates/selene_os/src/app_ingress.rs`, tool follow-up runs immediately after one PH1.X tool dispatch, but current repo truth does not prove a persisted pending deterministic time clarification carrier across the next user turn.
- In `/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs`, low-risk public deterministic tool answers can be recovered for typed-only time/weather, but current repo truth does not prove place-only follow-up replies are rebound to the pending time query.
- The live product screenshot proves the user-facing failure: `what is the time in Spain` clarifies, `Madrid` then fails with governance/runtime out-of-sync instead of completing Madrid time; `what is the time in Portugal` clarifies, and an unsupported follow-up can also fail through the same out-of-sync posture.
- Spain and Portugal are proof examples only. The repo law target is the global deterministic clarification rule: any clear one-timezone place answers directly; any ambiguous or multi-timezone place clarifies; the user's immediate clarification reply remains bound to the same pending deterministic time request in the same thread.
- No current repo truth justifies protected-action law widening, identity/access/simulation weakening, backend-route expansion, provider redesign, PH1.D redesign, or shell redesign for this seam.

## CURRENT

- Deterministic time currently works for some supported places.
- H362 improved deterministic global time resolution.
- H362 did not finish multi-turn clarification continuity for ambiguous or incomplete time queries.
- Incomplete time queries can still fall through to raw UTC/ISO output instead of asking a clarification.
- Ambiguous place time queries such as `what time is it in Spain` or `what time is it in Portugal` can ask a clean clarification.
- Follow-up replies to lawful time clarifications, such as `Madrid` after Spain or `Lisbon` after Portugal, can still lose pending time context and fall into governance/runtime failure instead of completing the same deterministic time request.
- The current proof is not global enough across single-timezone cities, single-timezone countries, multi-timezone countries, multi-timezone regions/states, ambiguous place names, and explicit IANA timezone input.
- No current proof justifies protected-action law widening.

## TARGET

- Any place query that maps cleanly to one timezone returns a clean direct answer.
- Any place query that is ambiguous or spans multiple timezones asks a clean clarification instead of guessing.
- The user's follow-up clarification stays bound to the same pending deterministic time request in the same thread.
- The follow-up reply completes the original deterministic time question instead of falling into governance/runtime failure.
- `what time is it in` returns a clean clarification such as `Which place do you mean?`
- Direct examples such as New York, Italy, Germany, Japan, Sydney, and `America/New_York` return clean final answers.
- Ambiguous or multi-timezone examples such as Portugal, Spain, United States, Australia, and Springfield clarify.
- Follow-up replies such as Lisbon or Azores, Madrid or Canary Islands, New York or Los Angeles, Sydney or Perth, and Springfield, Illinois or Springfield, Missouri complete the same pending request.
- Unsupported follow-up replies to a pending time clarification clarify cleanly instead of producing governance/runtime out-of-sync.
- The H362 provider stack is preserved: Google Time Zone primary, TimeZoneDB secondary fallback, host/system UTC current-time truth, and IANA timezone identity.
- Low-risk public deterministic time completion still does not require Voice ID.
- Protected memory, personalization, permissions, and protected actions remain behind current identity/access law.
- Existing lawful TTS playback remains available for completed answers.
- Final user output contains no raw ISO, raw provider payload, raw source dump, web-search dump, or governance out-of-sync message for lawful deterministic clarification follow-ups.

## GAP

This is not shell redesign, sidebar redesign, composer redesign, provider redesign, PH1.D redesign, generic backend expansion, or protected-action law widening.

The exact gap is deterministic clarification carry-over and same-thread follow-up completion for low-risk public time queries.

## In Scope

- Make incomplete deterministic time queries ask one clean missing-place clarification instead of returning UTC/raw ISO output.
- Make ambiguous or multi-timezone deterministic time queries ask one clean clarification instead of guessing.
- Preserve one pending low-risk public deterministic time clarification in the same thread using existing lawful runtime/session carriers.
- Bind the immediate follow-up reply to the pending deterministic time request when the reply is a lawful place/timezone clarification answer.
- Complete follow-ups such as Lisbon or Azores after Portugal, Madrid or Canary Islands after Spain, New York or Los Angeles after United States, Sydney or Perth after Australia, and Springfield, Illinois or Springfield, Missouri after Springfield through the same deterministic time lane.
- Prove the generic clarification law across single-timezone city, single-timezone country, multi-timezone country, multi-timezone region/state, ambiguous place name, and IANA timezone input categories.
- Preserve H362 Google Time Zone primary and TimeZoneDB secondary fallback behavior.
- Preserve PH1.N deterministic time classification.
- Preserve PH1.X dispatch to the deterministic time lane and clean final-answer rendering.
- Preserve adapter/runtime completion into the same visible conversation thread.
- Preserve low-risk public deterministic time completion without Voice ID.
- Preserve protected memory, personalization, permissions, and protected actions behind current identity/access law.
- Preserve existing lawful TTS playback for the completed answer.
- Add focused tests for incomplete query clarification, ambiguous query clarification, same-thread follow-up completion, and invalid follow-up clarification.

## Out of Scope

- Shell redesign.
- Sidebar redesign.
- Composer redesign.
- Provider swap or provider redesign.
- PH1.D redesign.
- PH1.D provider calls for deterministic time.
- Web-search fallback for deterministic time.
- Fake local answer lane.
- Backend route expansion.
- Protected-action law widening.
- Identity/access/simulation weakening.
- Wake-law changes.
- Session-law changes beyond preserving a lawful pending low-risk deterministic clarification carrier already available in current runtime/session truth.
- New secret ids.
- Weather work.
- General multi-turn assistant memory or personalization.
- False "works" claims without live desktop or exact repo-supported runtime proof.

## Files Allowed To Change

- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1n.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/app_ingress.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs`
- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`
- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`

Only if repo truth proves unavoidable:

- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1e.rs`

If and only if implementation succeeds and repo law requires landed-truth updates:

- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md`

## Files Not Allowed To Change

- PH1.D contract files.
- Provider-secret contract files.
- Backend route expansion files.
- DB wiring specs.
- iPhone source files.
- Unrelated desktop source files.
- Unrelated runtime law files.
- Weather provider files.
- Any file outside the allowed scope unless repo truth proves it is unavoidable, in which case stop and report first.

## Acceptance Standard

H363 is accepted only when all of the following are true:

- `what time is it in` produces a clean missing-place clarification, not UTC/raw ISO output.
- Single-timezone city queries produce clean direct answers.
- Single-timezone country queries produce clean direct answers.
- Multi-timezone country queries produce clean clarification prompts.
- Multi-timezone region/state queries produce clean clarification prompts.
- Ambiguous place names produce clean clarification prompts.
- IANA timezone input produces clean direct answers.
- Follow-up clarification replies complete the same pending deterministic time request in the same thread.
- `what time is it in Spain` produces a clean multi-timezone clarification, not a guessed answer.
- `Madrid` or `Canary Islands` immediately after the Spain clarification completes the same pending deterministic time request and returns a clean answer in the same thread.
- `what time is it in Portugal` produces a lawful clarification if required.
- `Lisbon` or `Azores` immediately after the Portugal clarification completes the same pending deterministic time request and returns a clean answer in the same thread.
- `what time is it in United States` clarifies, and `New York` or `Los Angeles` completes the same pending request.
- `what time is it in Australia` clarifies, and `Sydney` or `Perth` completes the same pending request.
- `what time is it in Springfield` clarifies, and `Springfield, Illinois` or `Springfield, Missouri` completes the same pending request.
- `what time is it in America/New_York` produces a clean direct answer.
- Unsupported clarification replies such as `Exotic` after Portugal clarify cleanly or ask for one supported option instead of producing governance/runtime out-of-sync.
- Follow-up completion does not require Voice ID because this remains a low-risk public deterministic time lane.
- Protected memory, personalization, permissions, and protected actions remain identity-gated.
- Google Time Zone remains primary and TimeZoneDB remains secondary fallback.
- No final answer exposes raw ISO strings, raw provider payloads, raw source dumps, web-search snippets, or `Retrieved at (unix_ms)`.
- No lawful deterministic time clarification follow-up produces `governance state is out of sync`.
- Existing lawful TTS can speak the completed final answer when enabled and lawful.

## Exact Test Commands

Preflight:

```bash
git -C /Users/selene/Documents/Selene-OS status --short
git -C /Users/selene/Documents/Selene-OS branch --show-current
git -C /Users/selene/Documents/Selene-OS rev-parse HEAD
```

Build check:

```bash
cargo check -p selene_engines -p selene_os -p selene_adapter
```

Existing H362 regression proof:

```bash
cargo test -p selene_engines h362 -- --nocapture
cargo test -p selene_os h362 -- --nocapture
cargo test -p selene_adapter h362 -- --nocapture
cargo test -p selene_adapter at_adapter_4 -- --nocapture
cargo test -p selene_os at_identity_posture_01_low_confidence_protected_voice_turn_fails_closed_with_explicit_low_confidence_response -- --nocapture
```

H363 implementation must add or identify exact runnable tests equivalent to:

```bash
cargo test -p selene_engines h363_time_query_missing_place_clarifies -- --nocapture
cargo test -p selene_os h363_time_missing_place_renders_clean_clarification -- --nocapture
cargo test -p selene_adapter h363_single_timezone_city_time_answers_directly -- --nocapture
cargo test -p selene_adapter h363_single_timezone_country_time_answers_directly -- --nocapture
cargo test -p selene_adapter h363_spain_madrid_clarification_followup_completes_same_thread -- --nocapture
cargo test -p selene_adapter h363_spain_canary_islands_clarification_followup_completes_same_thread -- --nocapture
cargo test -p selene_adapter h363_portugal_lisbon_clarification_followup_completes_same_thread -- --nocapture
cargo test -p selene_adapter h363_portugal_azores_clarification_followup_completes_same_thread -- --nocapture
cargo test -p selene_adapter h363_united_states_new_york_or_los_angeles_followup_completes_same_thread -- --nocapture
cargo test -p selene_adapter h363_australia_sydney_or_perth_followup_completes_same_thread -- --nocapture
cargo test -p selene_adapter h363_springfield_disambiguated_followup_completes_same_thread -- --nocapture
cargo test -p selene_adapter h363_iana_timezone_input_answers_directly -- --nocapture
cargo test -p selene_adapter h363_invalid_time_clarification_followup_does_not_governance_out_of_sync -- --nocapture
cargo test -p selene_adapter h363_low_risk_time_followup_does_not_require_voice_id -- --nocapture
cargo test -p selene_adapter h363_protected_actions_remain_identity_gated -- --nocapture
```

Exact repo-supported runtime proof must use the real adapter route or app path, for example:

```bash
cargo run -p selene_adapter --bin selene_adapter_http
```

Then submit the required live acceptance prompts through `/v1/voice/turn` or the macOS desktop app.

## Required Live Desktop Proof

The later implementation run must launch the actual macOS desktop app or exact repo-supported desktop runtime path and ask, in one same-thread conversation:

- `what time is it in`
- `what is the time in New York`
- `what is the time in Italy`
- `what is the time in Germany`
- `what is the time in Japan`
- `what is the time in Sydney`
- `what time is it in Spain`
- `Madrid`
- `what time is it in Spain`
- `Canary Islands`
- `what time is it in Portugal`
- `Lisbon`
- `what time is it in Portugal`
- `Azores`
- `what time is it in United States`
- `New York`
- `what time is it in United States`
- `Los Angeles`
- `what time is it in Australia`
- `Sydney`
- `what time is it in Australia`
- `Perth`
- `what time is it in Springfield`
- `Springfield, Illinois`
- `what time is it in Springfield`
- `Springfield, Missouri`
- `what time is it in America/New_York`

The proof must show:

- Incomplete questions clarify cleanly.
- Ambiguous questions clarify cleanly.
- Single-timezone cities answer directly.
- Single-timezone countries answer directly.
- Multi-timezone countries clarify and then complete after the user's follow-up.
- Multi-timezone regions/states clarify and then complete after the user's follow-up.
- Ambiguous place names clarify and then complete after the user's follow-up.
- IANA timezone input answers directly.
- The follow-up reply completes the same pending deterministic time request in the same thread.
- No raw ISO appears.
- No raw provider payload appears.
- No web-search dump appears.
- No `Retrieved at (unix_ms)` appears.
- No governance/runtime out-of-sync error appears for lawful deterministic time clarification follow-ups.
- Existing lawful TTS can speak completed answers when enabled and lawful.
- Screenshot proof of the visible main pane after completion is included when the actual app path is used.

If the actual app cannot be launched, the implementation run may use the exact repo-supported desktop runtime path only if it proves the same app ingress, adapter response, and desktop bridge completion path. It must stop and report instead of substituting unrelated unit tests for live acceptance.

## Stop Conditions

- Stop if deterministic clarification continuity requires protected-action law widening.
- Stop if follow-up completion requires backend route expansion.
- Stop if the only passing path would be fake local state instead of lawful same-thread carry-over.
- Stop if files outside the allowed scope become necessary.
- Stop if live desktop app or exact repo-supported runtime path cannot be launched/tested.
- Stop if Google Time Zone / TimeZoneDB logic must be rewritten instead of preserving the current H362 provider stack.
- Stop if low-risk public deterministic time follow-up completion still requires Voice ID or strong identity verification.
- Stop if protected memory, personalization, permissions, or protected actions would be weakened.
- Stop if web search or PH1.D would be needed to answer deterministic time.
- Stop if a new secret id would be required.

## Ledger Update Rule

If H363 later lands successfully, update only:

- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md`

The landed-truth update must state:

- H363 completed deterministic time clarification continuity for low-risk public time queries.
- H363 implemented the global deterministic clarification rule, not a Spain/Portugal-only exception.
- Incomplete time queries clarify instead of returning UTC/raw ISO.
- Ambiguous and multi-timezone time queries clarify instead of guessing.
- Immediate place follow-ups complete the pending deterministic time request in the same thread.
- The H362 provider stack remained intact: Google Time Zone primary and TimeZoneDB secondary.
- Low-risk public time still does not require Voice ID.
- Protected-action / identity / access / simulation law remained unchanged.
- No shell redesign, provider redesign, PH1.D redesign, backend route expansion, fake local answer lane, or web-search fallback was introduced.

If the implementation stops or is not accepted, do not update landed-truth docs.
