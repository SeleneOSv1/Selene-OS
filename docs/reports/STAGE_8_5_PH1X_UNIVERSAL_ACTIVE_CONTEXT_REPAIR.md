# Stage 8.5 PH1.X Universal Active Context Repair

Review date: 2026-05-17

## Executive Conclusion

This checkpoint removes the Stage 8.5 exact-phrase PH1.X repair direction and replaces it with a generalized PH1.X active-frame algorithm plus canonical contract fields for richer live conversation evidence.

JD explicitly deferred live Desktop/JD proof in this wrap-up pass so the repo can end clean before the next proper X build. Therefore this report does not claim full Stage 8.5 live acceptance.

Readiness: **NOT_READY_FOR_STAGE_9_FRESH_MEMORY_TRACE_EVAL** because live Desktop proof was intentionally deferred by JD. Automated owner proof is green and the repo is ready for the next PH1.X build from a clean tree.

## Files Changed

- `crates/selene_kernel_contracts/src/ph1x.rs`
- `crates/selene_os/src/ph1x.rs`
- `crates/selene_adapter/src/lib.rs`

No Desktop files were changed.

## Root Cause Fixed

The previous Stage 8.5 direction had drifted toward exact test-case handling for planning, writing, and protected follow-ups. That was the wrong shape. JD examples are regression cases, not production semantics.

The repaired direction is:

- PH1.X owns the live active frame.
- PH1.X extracts generalized turn features from normalized tokens and prior turn state.
- PH1.X scores continuation, correction, clarification, writing-artifact, planning, tool, topic-switch, and protected-boundary patterns.
- Adapter only bridges PH1.X output into the existing runtime route while legacy adapter context paths remain as compatibility.
- PH1.E receives resolved tool context only.
- Desktop remains render/transport only.

## Contract Foundation

`ActiveContextPacket` now carries optional universal active-frame fields:

- `user_goal`
- `current_plan`
- `open_question`
- `unresolved_decision`
- `prior_options_presented`
- `selected_option`
- `rejected_option`
- `comparison_set`
- `constraints`
- `user_preference_in_turn`
- `expected_answer_type`
- `last_clarification_question`
- `clarification_answer_target`
- `discourse_state`
- `topic_depth`
- `returnable_topic`
- `interruption_state`
- `speaker_continuity`
- `confidence_reason`
- `why_not_continue_reason`

These are nullable evidence fields. They do not force full Stage 9 behavior, and they do not create a second PH1.X packet.

## Algorithmic Generality Proof

Canonical owner: PH1.X.

Generalized algorithm:

1. Normalize the current user turn into reusable token features.
2. Read the active conversation frame from `ThreadState`, including active subject refs, pinned context refs, last route, last answer, pending clarification, and protected boundary markers.
3. Detect generic feature classes:
   - planning frame
   - selection/recommendation request
   - timing request inside planning
   - tool family mention
   - new entity/location fragment
   - clarification answer
   - correction target
   - writing artifact reference and modifier
   - protected confirmation
   - unrelated topic switch
4. Produce a resolved continuation query or decline continuation.
5. Update the PH1.X frame after each turn with generalized topic, constraint, option, artifact, clarification, protected-risk, and returnable-topic evidence.

Input features used:

- normalized token set
- generic planning/activity/tool/artifact/reference vocabulary
- prior route class
- active subject refs
- pinned context refs
- last user text and Selene answer
- pending clarification question and target
- protected-risk response markers

Output produced:

- PH1.X resolved continuation text for the existing bridge
- PH1.X active-frame refs in `ThreadState`
- canonical `ActiveContextPacket` fields for future evidence storage
- existing `HumanConversationDirective` compatibility through the current runtime route

Why this works beyond exact phrases:

- Planning tests cover destination plus constraints, then recommendation forms using city/area/place/base/timing language.
- Unseen paraphrase tests include Canada/snow/dining and Berlin time follow-up.
- Writing tests cover shorter, tighter, darker, warmer, shorten, and add-detail modifiers.
- Negative hijack tests prove identity, jokes, metadiscourse, and condition-like questions are not stolen by old tool context.

## Universal Active Frame Addendum Proof

Fields added/reused:

- Added the universal active-frame evidence fields listed above to `ActiveContextPacket`.
- Reused existing `ThreadState.subject_refs` and `ThreadState.pinned_context_refs` as the runtime active-frame storage surface.
- Reused existing PH1.X/adapter route classes for tool handoff instead of creating a second context route.

Fields deferred:

- Dedicated persisted rows for every universal field remain deferred to the later evidence/trace build. The current build exposes the canonical packet shape and runtime frame refs.

Equivalent existing fields reused:

- `ThreadState.subject_refs` stores normalized active topic, constraints, comparison set, clarification target, writing artifact, and protected boundary refs.
- `ThreadState.pinned_context_refs` stores frame refs that survive the current live context path.
- `LastTurnRouteClass` keeps tool family continuity.

Proof no Desktop semantic logic was added:

- No `apple/` or Desktop files changed.

Proof Adapter did not become the context brain:

- The adapter calls `ph1x_universal_active_context_followup_query` before legacy compatibility routing. The new semantic decision lives in `crates/selene_os/src/ph1x.rs`.

## Phrase-Patch Scan

Command:

```text
git diff --unified=0 | rg -n "which city|which areas|the time|Japan|Sydney|Melbourne|Brisbane|make it shorter|make it warmer|Mark|locked factory|Niseko|Hakuba|Nozawa|Sapporo|New York|payroll|Tim"
```

Classification:

- `TEST_FIXTURE_OK`: exact Japan, Sydney, Melbourne, Brisbane, New York, Mark, locked factory, Niseko, Hakuba, Sapporo, payroll, and Tim strings appear in regression tests and this report.
- `DOMAIN_VOCABULARY_OK`: `Time`/`ToolTime` hits are false positives from the `Tim` substring and canonical tool-family naming.
- `RETAINED_COMPATIBILITY_PATH`: existing adapter payroll and deterministic time/weather context vocabulary remains active compatibility from earlier stages.
- `PRODUCTION_PHRASE_PATCH_NOT_ALLOWED`: none added in the new PH1.X production algorithm.

Production-only PH1.X check over the new algorithm showed no exact JD example strings in production logic. Exact examples are confined to tests/report material.

## Old Shortcut Path Classification

Scanned affected owners for `contains`, `starts_with`, `ends_with`, exact equality checks, deterministic context names, and shortcut/fallback markers.

- `crates/selene_os/src/ph1x.rs` new Stage 8.5 token-feature logic: `DOMAIN_VOCABULARY_OK` / `ALGORITHM_FEATURE_MATCH_OK`. These are category vocabularies and token rules, not exact prompt branches.
- `crates/selene_adapter/src/lib.rs::deterministic_active_context_followup_query`: `RETAINED_COMPATIBILITY_PATH`. Still active for Stage 1-8 route compatibility; PH1.X is now called first.
- `crates/selene_adapter/src/lib.rs::deterministic_weather_context_followup_query`: `RETAINED_COMPATIBILITY_PATH`. Still active for H363/H364/H409 compatibility; future retirement requires PH1.X to fully replace that route and carry all legacy tests.
- Adapter payroll classification logic: `RETAINED_COMPATIBILITY_PATH`. It remains because protected fail-closed behavior is active and tested. Future owner cleanup should move remaining semantic classification out of adapter once the protected owner path is ready.
- Exact test scenario strings in `#[cfg(test)]`: `TEST_FIXTURE_OK`.

No Desktop shortcut path was added.

## Automated Proof

Targeted:

- `cargo test -p selene_os stage8_5 -- --test-threads=1` — passed, 19 tests.
- `cargo test -p selene_adapter stage8_5 -- --test-threads=1` — passed.
- `cargo test -p selene_kernel_contracts ph1x -- --test-threads=1` — passed, 40 tests.
- `cargo test -p selene_adapter h409_live_time_context_weather_like_followups_route_to_weather -- --test-threads=1` — passed.

Full package checks:

- `cargo test -p selene_kernel_contracts -- --test-threads=1` — passed, 396 tests.
- `cargo test -p selene_storage -- --test-threads=1` — passed.
- `cargo test -p selene_adapter -- --test-threads=1` — passed, 468 library tests with 4 ignored live-provider tests plus adapter integration/bin tests.
- `cargo test -p selene_os -- --test-threads=1` — passed, 1701 library tests plus bin tests.
- `cargo test -p selene_engines -- --test-threads=1` — passed, 665 tests with 12 ignored live-provider tests.
- `cargo check` — passed.

## Ignored `selene_engines` Tests

The exact requested command:

```text
cargo test -p selene_engines -- --list | rg "ignored|Ignore|ignored"
```

prints one non-ignored test whose name contains `ignored`:

- `ph1n::tests::run2_send_link_transcript_tenant_hint_mismatch_is_ignored`

The actual ignored tests, from `cargo test -p selene_engines -- --ignored --list`, are:

1. `ph1e::tests::h383_live_brave_web_query_returns_verified_sources`
2. `ph1e::tests::h389_live_brave_image_provider_approval_maps_real_metadata_without_secret_leak`
3. `ph1e::tests::h390_live_brave_image_display_eligibility_maps_real_metadata_without_display`
4. `ph1e::tests::h391_live_brave_image_provider_policy_maps_real_metadata_without_display`
5. `ph1e::tests::h392_live_brave_source_link_card_maps_real_metadata_without_image_display`
6. `ph1e::tests::h393_live_brave_source_link_click_safety_maps_real_metadata_without_auto_open`
7. `ph1e::tests::h394_live_gdelt_doc_api_returns_bounded_corroboration_metadata`
8. `ph1e::tests::h395_live_gdelt_rust_transport_records_transport_outcome`
9. `ph1e::tests::h399_live_gdelt_explicit_proxy_route_records_proxy_outcome`
10. `ph1e::tests::h400_live_gdelt_proxy_tls_connect_records_connect_outcome`
11. `ph1e::tests::h401_live_gdelt_proxy_protocol_route_records_final_blocker`
12. `ph1e::tests::h402_live_gdelt_socks_tls_phase_records_exact_blocker`

Why ignored:

- All twelve are PH1.E live provider/network/proxy proofs.
- Brave cases require a real Brave Search secret and live network access.
- GDELT cases require live provider/proxy network conditions.
- They are not PH1.X live-context tests.

Stage 8.5 relevance:

- None directly block PH1.X active-context completion.
- They may need manual opt-in/live execution for PH1.E provider acceptance, not for this PH1.X cleanup checkpoint.

## Live Desktop Proof

Live Desktop/JD smoke was **not run** in this wrap-up pass because JD explicitly instructed:

> we will not do life tests as we have some more to build - what you should do is rap things up and end with a clean tree so we can start building X properly

No current app provenance claim is made in this report.

## Final Status

Stage 8.5 automated PH1.X cleanup proof: passed.

Full Stage 8.5 live acceptance: not claimed.

Readiness: **NOT_READY_FOR_STAGE_9_FRESH_MEMORY_TRACE_EVAL** until the required live Desktop proof is run after the next X build.
