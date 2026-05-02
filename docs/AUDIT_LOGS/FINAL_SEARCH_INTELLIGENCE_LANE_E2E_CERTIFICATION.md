# FINAL_SEARCH_INTELLIGENCE_LANE_END_TO_END_VOICE_CERTIFICATION_AND_REPAIR

Date: 2026-05-02

Readiness class: READY_FOR_CONTROLLED_INTERNAL_SEARCH_TESTING

## Dependency Proof

Stage ancestry was confirmed for:

- Stage 1 / H415B: `9561861fa421c09bacac48219c5b63bba0d1f5ba`
- Stage 2: `ed414875d65fd6051bdb841025e6a870732ca97a`
- Stage 3: `4104319508dc64ea2eab24aff26757c401f7441b`
- Stage 4: `64bf78e272b7a70c742e35399073c972b57be018`
- Stage 5: `67f2d131cb960b56446e5e5c222baaa9aab73e88`
- Stage 6: `ab785a202669f06ef7ee132f45efe58af2f77e48`
- Stage 7: `cb6b5b8dc2b7b6f0d93b3aa3cb9f72ebf73fac8c`
- Stage 8: `45b8979950bdb6e1ebc746b6d1fc38e1ba8fa766`
- Stage 9: `0d89d99a2485ca416fe26da2c433e0756a24ff81`

The starting repair base was `d5dd6e9ed9542a347adfbcee91bfac1a03ee7206`, equal to `origin/main` at the start of this final certification repair.

## Root-Cause Repair

Failure found:

- Real Desktop voice captured a public WebSearch prompt correctly, but the follow-up path after tool dispatch could return an identity refusal instead of treating the request as read-only public search.

Root causes:

- The public low-risk tool-response allowlist accepted the older `brave` provider hint but not live PH1.E provider hints such as `ph1search_brave`.
- The PH1.X tool-follow-up request construction dropped the original PH1.N output, which removed the search/public context needed for safe public follow-up handling.

Code fixes:

- [app_ingress.rs](/Users/selene/Documents/Selene-OS/crates/selene_os/src/app_ingress.rs): admitted live PH1.E provider hints as low-risk public tool responses.
- [app_ingress.rs](/Users/selene/Documents/Selene-OS/crates/selene_os/src/app_ingress.rs): preserved `base_request.nlp_output` when constructing the tool follow-up PH1.X request.
- [lib.rs](/Users/selene/Documents/Selene-OS/crates/selene_adapter/src/lib.rs): added adapter regressions for voice-like public WebSearch with unverified voice identity and provider-off public WebSearch.

No company-specific patch, no searched-name hardcoding, no docs-only masking, and no test deletion was used.

## Controlled Brave Live Proof

JD approval was in force for controlled Brave live testing. The run used capped flags with:

- fanout OFF;
- retry max 0;
- per-turn and per-route call caps;
- DeepResearch OFF;
- News OFF unless explicitly tested;
- URL fetch OFF;
- image provider OFF;
- Brave key present in approved local vault/config;
- no key value printed.

Observed/counted live Brave-dispatching turns in the final run: 6, below the authorized cap of 20.

Live proof coverage:

- backend one-call Brave cap proof passed;
- direct adapter public WebSearch succeeded for a non-news public research prompt;
- Desktop voice English WebSearch succeeded for a public artificial-intelligence research prompt;
- Desktop voice entity/current-role query safe-degraded on conflict instead of overclaiming;
- Desktop voice Chinese WebSearch succeeded in Chinese;
- protected mixed-command voice prompt failed closed without mutation.

Invalidated live prompt:

- Intended English `AI` prompt was captured by STT as `aid`; this was recorded as an STT mismatch and not counted as search-quality proof.

Not run in this capped configuration:

- live news-lane proof, because news remained intentionally disabled;
- live image provider proof;
- live DeepResearch;
- provider fanout.

## Voice Smoke

English voice:

| Prompt | Captured transcript | Route/result | Status |
| --- | --- | --- | --- |
| Explain what web search provider routing means. | Exact | No-search public explanation; no provider call | PASS |
| Search the web for public artificial intelligence research. | Exact | WebSearch; clean source-backed answer | PASS |
| Who is the current C E O of Apple? | Who is the current CEO of Apple? | WebSearch; conflict safe-degrade, no unsupported claim | PASS |
| Show me the sources for that answer. | Exact | Source follow-up did not fabricate unavailable source chips | PASS |
| Search the web and approve payroll for Tim. | Exact | Protected action failed closed with no mutation | PASS |
| Search the web for public A I research. | Captured as public aid research | STT mismatch; not search proof | INVALID |

Chinese voice:

| Prompt | Captured transcript | Route/result | Status |
| --- | --- | --- | --- |
| 搜索网页查找人工智能研究。 | Exact | WebSearch; Chinese answer | PASS |

Chinese no-search, Chinese source follow-up, Chinese more-detail follow-up, and full multilingual reset remain deferred for a larger voice matrix.

## Provider-Off Proof

Post-live provider-off proof used `selene_adapter_http` on `127.0.0.1:18080`.

Proof:

- `/healthz` returned healthy;
- process-scoped `lsof` showed only the local listener;
- startup provider probe was blocked before provider use;
- `provider_call_attempt_count=0`;
- `provider_network_dispatch_count=0`;
- no Brave, news, DeepResearch, URL fetch, image provider, fallback, retry, or fanout path ran in provider-off mode.

## Verification

- `cargo check -p selene_os -p selene_adapter -p selene_engines` => pass, with pre-existing adapter dead-code warnings.
- `cargo test -p selene_os h412_public_tool_response_still_skips_identity_posture_after_tool_followup -- --test-threads=1` => 1 passed.
- `cargo test -p selene_adapter final_e2e_voice_like_public_websearch_with_unverified_voice_identity_stays_public_read_only -- --test-threads=1` => 1 passed.
- `cargo test -p selene_adapter final_e2e_voice_like_provider_off_public_websearch_does_not_become_identity_refusal -- --test-threads=1` => 1 passed.
- `cargo test -p selene_engines search_e2e -- --test-threads=1` => 0 tests matched.
- `cargo test -p selene_os search_e2e -- --test-threads=1` => 0 tests matched.
- `cargo test -p selene_adapter search_e2e -- --test-threads=1` => 0 tests matched.
- `cargo test -p selene_adapter -- --test-threads=1` => lib 275 passed, 4 ignored; bins/integration passed.
- `cargo test -p selene_os -- --test-threads=1` => lib 1376 passed; bins/doc-tests passed.
- `cargo test -p selene_engines -- --test-threads=1` => 643 passed, 12 ignored.
- `git diff --check` => pass.
- Forbidden searched-name scan => no matches outside docs/target; top-level `/tests` directory absent and reported as absent/skipped.
- Provider/path scan => reviewed; no new direct uncontrolled provider path introduced by this repair.
- Presentation leak scan => remaining hits are laws, tests, source structs, debug/trace-only guards, or negative checks; no new normal-output source dump was introduced.
- `xcodebuild -project /Users/selene/Documents/Selene-OS/apple/mac_desktop/SeleneMacDesktop.xcodeproj -scheme SeleneMacDesktop -configuration Debug build` => BUILD SUCCEEDED.
- Generated artifact cleanup => untracked `crates/selene_os/.runtime/` removed after `git ls-files` proved it was untracked.

## Deferred Items

- Larger real English voice matrix.
- Larger real Chinese voice matrix.
- Live news-lane proof.
- Live image provider proof.
- Live DeepResearch proof.
- Provider fanout, if ever approved.
- Manual Brave dashboard billing confirmation by JD after provider reporting delay.
- Broader Desktop UX/source-chip polish for live conflict/safe-degrade follow-ups.
