# H364 - PH1.E Global Weather Local Place Resolution, Clarification Continuity, And Desktop Completion Build Plan

## Purpose

H364 is the narrow doc-only build plan after H363.

H361 established the lawful weather provider foundation: Tomorrow.io is primary, WeatherAPI is secondary fallback, and both provider secret ids are already present. H362 and H363 advanced deterministic time, including global time resolution and same-thread clarification continuity. Weather now needs the same clean-answer discipline, but weather does not follow the exact same direct-answer law as time.

Weather is local. Temperature, rain, sun, wind, and humidity can vary inside the same country, region, city, or district. Selene must answer directly only when the user asks for a clear city or lawfully resolved local place. Selene must clarify when the request is a country, broad region, state/province, or ambiguous place name. The user's clarification follow-up must remain bound to the same pending weather request in the same visible conversation thread.

This seam is not shell redesign, sidebar redesign, composer redesign, PH1.D redesign, generic backend expansion, protected-action law widening, or fake local weather behavior.

## Authority

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

- Preflight for this doc-only run started clean on `main` at `ebf17389ac2415072967d39c2c4a71ba42f01ca0`.
- Existing H-series repo truth already contains H363 at `/Users/selene/Documents/Selene-OS/docs/PHASE_PLANS/PHASE_H_IMPLEMENTATION/H363_PH1E_DETERMINISTIC_TIME_CLARIFICATION_CONTINUITY_AND_SAME_THREAD_FOLLOWUP_COMPLETION_BUILD_PLAN.md`.
- Therefore the next free implementation id is H364, not H363.
- `/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/provider_secrets.rs` already includes `tomorrow_io_api_key` and `weather_api_key`.
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/web_search_plan/realtime/adapters/weather.rs` resolves Tomorrow.io first and WeatherAPI second when the primary is unconfigured, upstream-failed, timed out, or unavailable.
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/web_search_plan/realtime/mod.rs` already has Tomorrow.io and WeatherAPI endpoint configuration.
- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1n.rs` already classifies weather questions as `WeatherQuery`.
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs` already dispatches `WeatherQuery` to `ToolName::Weather`.
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/app_ingress.rs` already contains a weather completion seam that extracts a place, clarifies country-level weather via `weather_country_ambiguity`, calls realtime weather, normalizes Tomorrow.io and WeatherAPI payloads, and renders clean weather sentences.
- Current weather ambiguity handling is not globally sufficient: it mainly handles country-level ambiguity and does not yet prove broad regions, ambiguous place names, or same-thread follow-up completion.
- Current proof is not enough to claim global weather works perfectly for every country and local area.
- No repo truth justifies protected-action law widening for low-risk public weather answers.

## CURRENT

- Tomorrow.io is already primary.
- WeatherAPI is already secondary fallback.
- Required secret ids already exist:
  - `tomorrow_io_api_key`
  - `weather_api_key`
- Weather provider-order wiring exists.
- Weather classification and dispatch exist.
- Weather app-ingress completion and clean sentence rendering exist for some supported city/place cases.
- Country-level weather ambiguity handling exists for some countries, but it is not a complete global place-resolution law.
- Broad regions, states/provinces, ambiguous place names, and follow-up clarification completion are not fully proven end-to-end.
- Current proof is not enough to say weather works globally for every lawfully resolvable local place.
- No current proof justifies protected-action law widening.

## TARGET

- City or clearly normalized local place weather queries answer directly.
- Country, large region, broad area, state/province, or ambiguous place-name weather queries ask a clean clarification.
- The user's follow-up clarification remains bound to the same pending weather request in the same thread.
- Follow-up replies such as `Lisbon`, `Madrid`, or `Springfield, Illinois` complete the original pending weather request instead of starting an unrelated turn or failing governance/runtime state.
- Tomorrow.io is used first.
- WeatherAPI is used only as fallback when Tomorrow.io is unavailable, fails, times out, is unconfigured, or returns no lawful result.
- Both providers normalize into one Selene weather evidence/result shape.
- Final weather answers are clean, concise, and human-readable.
- Final weather answers can include temperature, condition, precipitation/rain when available, humidity when available, and wind when available.
- Final weather answers do not expose raw provider payloads, raw JSON, raw source dumps, raw API errors, web-search snippets, or `Retrieved at (unix_ms)`.
- Completed answers return through app ingress, adapter response, desktop runtime bridge, and the visible main pane.
- Existing lawful TTS can speak completed weather answers when enabled.
- Protected memory, personalization, permissions, and protected actions remain behind current identity/access law.

## GAP

This is not shell redesign, sidebar redesign, composer redesign, PH1.D answer-lane redesign, generic backend expansion, wake-law change, session-law change, identity/access/simulation weakening, or protected-action law widening.

The exact gap is global weather local-place resolution, broad/ambiguous-place clarification, provider fallback discipline, clean final-answer rendering, same-thread weather clarification carry-over, and live desktop completion proof.

The plan is not "country weather." The lawful global standard is: accurate weather for any lawfully resolvable local place worldwide, and clean clarification everywhere else.

## In Scope

- Preserve PH1.N `WeatherQuery` classification.
- Preserve PH1.X dispatch of `WeatherQuery` to `ToolName::Weather`.
- Preserve Tomorrow.io as primary.
- Preserve WeatherAPI as secondary fallback only.
- Use existing weather provider secret ids only.
- Normalize Tomorrow.io and WeatherAPI responses into one Selene weather result/evidence shape.
- Require local-place resolution before provider execution for direct answers.
- Allow provider-side resolution only for clear city/local-place queries where repo truth proves the provider can lawfully resolve the location.
- Clarify countries, broad regions, states/provinces, and ambiguous place names instead of silently guessing.
- Carry pending weather clarification state through the same thread.
- Complete follow-up clarification replies against the original pending weather request.
- Render clean final weather answers.
- Preserve visible main-thread completion in the desktop app.
- Preserve existing lawful TTS playback for completed weather answers.
- Add tests proving no raw provider payload or web-search dump becomes the final user answer.
- Preserve protected-action fail-closed and simulation-first law.

## Out of Scope

- No fake local weather answer lane.
- No country-wide weather guessing.
- No broad-region weather guessing.
- No ambiguous place-name guessing.
- No web-search fallback for weather unless the user explicitly asks to search the web.
- No PH1.D redesign.
- No protected-action law widening.
- No identity/access/simulation weakening.
- No wake-law changes.
- No session-law changes.
- No shell redesign.
- No sidebar redesign.
- No composer redesign.
- No generic backend expansion.
- No iPhone implementation work.
- No DB wiring spec rewrite.
- No new secret ids.
- No false "works globally" claim without live desktop proof.

## Files Allowed To Change

- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1e.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1n.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/app_ingress.rs`
- `/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs`
- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`
- `/Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`

If and only if implementation succeeds and repo law requires landed-truth updates:

- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md`
- `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md`

## Files Not Allowed To Change

- PH1.D contract files, unless repo truth proves unavoidable; if so, stop and report first.
- Backend route expansion files, unless repo truth proves unavoidable; if so, stop and report first.
- DB wiring specs.
- iPhone source files.
- Unrelated desktop source files.
- Unrelated runtime law files.
- Provider-secret contract files, because the required secret ids already exist.
- Any file outside the allowed scope unless repo truth proves unavoidable; if so, stop and report first.

## Acceptance Standard

- `what is the weather in New York` returns a clean final answer in the live macOS desktop app.
- `what is the weather in Tokyo` returns a clean final answer in the live macOS desktop app.
- `what is the weather in Sydney` returns a clean final answer in the live macOS desktop app.
- `what is the weather in Lisbon` returns a clean final answer in the live macOS desktop app.
- `what is the weather in Madrid` returns a clean final answer in the live macOS desktop app.
- `what is the weather in Japan` clarifies cleanly instead of guessing country-wide weather.
- `what is the weather in Portugal` clarifies cleanly instead of guessing country-wide weather.
- `what is the weather in Spain` clarifies cleanly instead of guessing country-wide weather.
- `what is the weather in United States` clarifies cleanly instead of guessing country-wide weather.
- `what is the weather in Springfield` clarifies cleanly instead of guessing the wrong Springfield.
- `Lisbon` immediately after the Portugal clarification completes the same pending weather request in the same thread.
- `Madrid` immediately after the Spain clarification completes the same pending weather request in the same thread.
- `Springfield, Illinois` immediately after the Springfield clarification completes the same pending weather request in the same thread.
- Tomorrow.io is attempted first for direct supported weather requests.
- WeatherAPI is used only as fallback when Tomorrow.io is unconfigured, upstream-failed, timed out, unavailable, or returns no lawful result.
- Final user answers do not expose raw provider JSON, raw provider payloads, raw source dumps, raw API errors, HTML entities, web-search snippets, or `Retrieved at (unix_ms)`.
- Provider-unavailable weather fails closed cleanly without fake weather and without dumping raw payloads.
- Completed weather answers return to the same visible main conversation thread.
- Existing lawful TTS can speak completed weather answers when enabled.
- Protected-action, identity/access, and simulation fail-closed behavior are not widened.

## Exact Test Commands

```sh
git -C /Users/selene/Documents/Selene-OS status --short
git -C /Users/selene/Documents/Selene-OS branch --show-current
git -C /Users/selene/Documents/Selene-OS rev-parse HEAD
cargo test -p selene_kernel_contracts provider_secret_ids_are_roundtrippable
cargo test -p selene_os web_search_plan::realtime
cargo test -p selene_engines at_n_40_weather_in_tokyo_is_weather_query
cargo test -p selene_os weather
cargo test -p selene_adapter weather
xcodebuild -project /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop.xcodeproj -scheme SeleneMacDesktop -configuration Debug build
```

The implementation run must add or update exact tests proving:

- Clear city/local-place weather requests complete with clean final answers.
- Country weather requests clarify.
- Broad region/state/province weather requests clarify.
- Ambiguous place names clarify.
- Weather clarification follow-ups complete the same pending request in the same thread.
- Tomorrow.io is selected before WeatherAPI.
- WeatherAPI fallback is used only when Tomorrow.io is unconfigured, upstream-failed, timed out, unavailable, or returns no lawful result.
- Raw provider payloads do not become final answers.
- Web search is not used for normal weather queries unless explicitly requested.
- Provider-unavailable weather fails closed cleanly.

## Required Live Desktop Proof

The implementation run must launch the actual macOS desktop app or exact repo-supported desktop runtime path and prove:

1. Ask `what is the weather in New York`.
2. Ask `what is the weather in Tokyo`.
3. Ask `what is the weather in Sydney`.
4. Ask `what is the weather in Lisbon`.
5. Ask `what is the weather in Madrid`.
6. Prove all direct city/place examples return clean final answers in the visible main pane.
7. Ask `what is the weather in Japan`.
8. Ask `what is the weather in Portugal`.
9. Ask `what is the weather in Spain`.
10. Ask `what is the weather in United States`.
11. Ask `what is the weather in Springfield`.
12. Prove all country/broad/ambiguous examples clarify cleanly instead of guessing.
13. Ask `what is the weather in Portugal`, then `Lisbon`, and prove the follow-up completes the same pending weather request in the same thread.
14. Ask `what is the weather in Spain`, then `Madrid`, and prove the follow-up completes the same pending weather request in the same thread.
15. Ask `what is the weather in Springfield`, then `Springfield, Illinois`, and prove the follow-up completes the same pending weather request in the same thread.
16. Prove Tomorrow.io is attempted first.
17. Prove WeatherAPI is used only as fallback.
18. Prove no raw provider payload appears.
19. Prove no web-search dump appears.
20. Prove desktop bridge logs show request completion.
21. Prove the same answer can be spoken through existing TTS when lawful and enabled.
22. Include screenshot proof of the visible main pane after completion.

If the actual desktop app cannot be launched/tested, stop and report instead of substituting unit tests for live acceptance.

## Stop Conditions

- Stop if the next free id is not H364.
- Stop if the tree is dirty before implementation.
- Stop if truthful weather completion requires provider use outside Tomorrow.io primary plus WeatherAPI secondary.
- Stop if required provider secret ids are missing.
- Stop if new secret ids would be required.
- Stop if lawful place normalization/geocoding is missing and the implementation cannot prove provider-side clear local-place resolution is lawful for direct city/place queries.
- Stop if global weather completion requires backend route expansion.
- Stop if desktop completion requires protected-action authority.
- Stop if clarification follow-up completion requires protected-action law widening.
- Stop if the result can only pass by fabricating a local desktop answer.
- Stop if the only passing path would be raw provider dump, fake weather, or web-search fallback.
- Stop if ambiguous country, region, state/province, or place-name handling would require guessing silently.
- Stop if live desktop app cannot be launched/tested.
- Stop if TTS playback requires new runtime authority instead of existing carriers.
- Stop if identity/access/simulation law changes appear necessary.

## Ledger Update Rule

Update `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md` and `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md` only after successful implementation and live desktop acceptance proof, if repo law requires landed-truth updates.
