# H361 - PH1.E Global Deterministic Weather Provider Normalization And Desktop Completion Build Plan

## Purpose

H361 is the narrow build-plan run after H360.

The weather lane now has the lawful provider foundations needed for real execution: Tomorrow.io is wired as the primary realtime weather provider, WeatherAPI is wired as the secondary fallback, and both provider secret ids are present in the Selene device vault. H361 must turn that provider-order wiring into a completed Selene weather path: normalized weather evidence, clean PH1.X final answers, existing app ingress / adapter / desktop bridge completion, visible main-pane output, and lawful TTS playback when already enabled.

Weather is not the same as time. Time can often answer a country directly when the timezone rule is unambiguous. Weather is place-specific and country-level weather is often ambiguous, so Selene must clarify broad country or region weather requests instead of guessing a location.

This seam is not shell redesign, sidebar redesign, composer redesign, PH1.D answer-lane redesign, generic backend expansion, or protected-action law widening.

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

- The highest committed H-series plan before this document is H360, so the next free implementation id is H361.
- `/Users/selene/Documents/Selene-OS/crates/selene_kernel_contracts/src/provider_secrets.rs` now includes `weather_api_key` and `tomorrow_io_api_key`.
- The Selene device vault has both `tomorrow_io_api_key` and `weather_api_key` present.
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/web_search_plan/realtime/adapters/weather.rs` tries Tomorrow.io first and WeatherAPI second when the primary is unconfigured, upstream-failed, timed out, or unavailable.
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/web_search_plan/realtime/adapters/weather.rs` normalizes provider payloads into one Selene realtime payload shape containing `retrieved_at_ms`, `trust_tier`, `provider`, `query`, and `provider_payload`.
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/web_search_plan/realtime/mod.rs` has default provider endpoints for Tomorrow.io and WeatherAPI.
- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1n.rs` already classifies weather questions as `WeatherQuery`.
- `/Users/selene/Documents/Selene-OS/crates/selene_os/src/ph1x.rs` already dispatches `WeatherQuery` to `ToolName::Weather`.
- `/Users/selene/Documents/Selene-OS/crates/selene_engines/src/ph1e.rs` still fails closed for `ToolName::Weather` with `weather_provider_not_wired`, so the provider-order wiring is not yet connected to the PH1.E weather tool completion lane.
- Current proof is not yet enough to say weather works globally for supported places.
- No repo truth justifies protected-action law widening for low-risk public weather answers.

## CURRENT

- Tomorrow.io is already primary.
- WeatherAPI is already secondary.
- Both API keys are already present in the Selene vault.
- Provider-order wiring exists in the realtime adapter.
- Provider outputs are partially normalized at the realtime adapter boundary.
- PH1.N weather classification exists.
- PH1.X weather dispatch exists.
- PH1.E still does not complete weather through the live weather provider lane.
- Global weather place normalization, ambiguity handling, clean final answer behavior, and live desktop completion are not yet fully proven end-to-end.
- No current proof justifies protected-action law widening.

## TARGET

- Supported city/place weather queries complete end-to-end in the live macOS desktop app.
- The final answer is clean, concise, and human-readable.
- The final answer is not a raw provider payload.
- The final answer is not a web-search dump.
- Tomorrow.io is used first.
- WeatherAPI is used only as fallback when Tomorrow.io is unconfigured, upstream-failed, timed out, or unavailable.
- Both provider outputs normalize into one Selene weather result/evidence shape.
- Ambiguous country or region weather queries clarify cleanly instead of guessing.
- Completed weather answers return through existing app ingress, adapter response, desktop runtime bridge, and visible main pane.
- Existing lawful TTS can speak the completed answer when enabled.
- No new protected-action authority is added.

## GAP

This is not shell redesign, sidebar redesign, composer redesign, PH1.D answer-lane redesign, generic backend expansion, wake-law change, session-law change, identity/access/simulation weakening, or protected-action law widening.

The exact gap is global weather place normalization, provider fallback, clean final-answer rendering, and live desktop completion proof.

## In Scope

- Connect PH1.E `ToolName::Weather` to the existing realtime weather provider lane.
- Preserve PH1.N `WeatherQuery` classification.
- Preserve PH1.X weather dispatch while adding clean weather final-answer rendering if needed.
- Normalize Tomorrow.io and WeatherAPI provider payloads into one Selene weather result/evidence shape.
- Use Tomorrow.io as the primary provider for supported weather queries.
- Use WeatherAPI only as the secondary fallback when Tomorrow.io is unconfigured, upstream-failed, timed out, or unavailable.
- Add global weather location normalization for supported city/place queries.
- Add clean clarification for ambiguous country/region requests.
- Return completed weather results through existing app ingress, adapter response, desktop runtime bridge, and visible main pane.
- Preserve existing lawful TTS playback for completed weather answers.
- Add tests proving no raw provider payload or web-search dump becomes the final answer.
- Preserve protected-action fail-closed and simulation-first law.

## Out of Scope

- No fake local weather answer lane.
- No web-search fallback for deterministic weather unless the user explicitly asks to search the web.
- No protected-action law widening.
- No identity/access/simulation weakening.
- No wake-law changes.
- No session-law changes.
- No shell redesign.
- No sidebar redesign.
- No composer redesign.
- No PH1.D provider or answer-lane redesign.
- No generic backend expansion.
- No iPhone implementation work.
- No DB wiring spec rewrite.
- No false "works" claim without live desktop proof.

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
- Any file outside the allowed scope unless repo truth proves unavoidable; if so, stop and report first.

## Acceptance Standard

- `what is the weather in New York` returns a clean final answer in the live macOS desktop app.
- `what is the weather in Tokyo` returns a clean final answer in the live macOS desktop app.
- `what is the weather in Sydney` returns a clean final answer in the live macOS desktop app.
- `what is the weather in Japan` clarifies cleanly instead of guessing one country-wide weather answer.
- Tomorrow.io is attempted first for supported weather queries.
- WeatherAPI is used only as fallback when Tomorrow.io is unconfigured, upstream-failed, timed out, or unavailable.
- Final user answers do not expose raw provider payloads, raw source dumps, raw JSON, HTML entities, or web-search snippets.
- Explicit web-search requests may still use web search as evidence, but normal weather queries must not become web-search dumps.
- Completed weather answers return to the same main conversation thread.
- Existing lawful TTS can speak the completed answer when enabled.
- Protected-action, identity/access, and simulation fail-closed behavior are not widened.

## Exact Test Commands

```sh
git -C /Users/selene/Documents/Selene-OS status --short
git -C /Users/selene/Documents/Selene-OS branch --show-current
git -C /Users/selene/Documents/Selene-OS rev-parse HEAD
cargo test -p selene_kernel_contracts provider_secret_ids_are_roundtrippable
cargo test -p selene_os web_search_plan::realtime
cargo test -p selene_engines at_n_40_weather_in_tokyo_is_weather_query
cargo test -p selene_engines at_e_weather
cargo test -p selene_os weather
cargo test -p selene_adapter weather
xcodebuild -project /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop.xcodeproj -scheme SeleneMacDesktop -configuration Debug build
```

The implementation run must add exact tests proving:

- Tomorrow.io is selected before WeatherAPI.
- WeatherAPI fallback is used only when Tomorrow.io is unconfigured, upstream-failed, timed out, or unavailable.
- New York, Tokyo, and Sydney weather produce clean final answers.
- Japan or another broad country/region weather query clarifies instead of guessing.
- Raw provider payloads do not become the final answer.
- Web search is not used for normal weather queries unless explicitly requested.
- Provider-unavailable weather fails closed cleanly.

## Required Live Desktop Proof

The implementation run must launch the actual macOS desktop app or exact repo-supported desktop runtime path and prove:

1. Ask `what is the weather in New York`.
2. Ask `what is the weather in Tokyo`.
3. Ask `what is the weather in Sydney`.
4. Prove all supported city/place examples return clean final answers in the visible main pane.
5. Ask `what is the weather in Japan`.
6. Prove Selene clarifies instead of guessing a country-wide weather answer.
7. Prove no raw provider payload appears.
8. Prove no web-search dump appears.
9. Prove desktop bridge logs show request completion.
10. Prove the same answer can be spoken through existing TTS when lawful and enabled.
11. Include screenshot proof of the visible main pane after completion.

If the actual desktop app cannot be launched/tested, stop and report instead of substituting unit tests.

## Stop Conditions

- Stop if the next free id is not H361.
- Stop if the tree is dirty before implementation.
- Stop if truthful weather completion requires backend route expansion.
- Stop if global place normalization requires files outside the allowed scope.
- Stop if fallback logic requires widening protected-action authority.
- Stop if Tomorrow.io and WeatherAPI credentials are unavailable from the Selene vault or lawful environment config.
- Stop if the only passing path would be a raw provider dump or web-search fallback instead of normalized weather handling.
- Stop if ambiguous country/region weather handling would require guessing silently.
- Stop if live desktop app cannot be launched/tested.
- Stop if TTS playback requires new runtime authority instead of existing carriers.
- Stop if identity/access/simulation law changes appear necessary.

## Ledger Update Rule

Update `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_PLAN.md` and `/Users/selene/Documents/Selene-OS/docs/MASTER_BUILD_COMPLETION_LEDGER.md` only after successful implementation and live desktop acceptance proof, if repo law requires landed-truth updates.
