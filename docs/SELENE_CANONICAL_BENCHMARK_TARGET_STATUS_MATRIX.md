# Selene Canonical Benchmark Target Status Matrix

Status: STAGE1_BENCHMARK_OWNERSHIP_ARTIFACT
Date: 2026-05-03

The canonical numeric targets are present in `docs/SELENE_CANONICAL_MASTER_BUILD_PLAN.md` as `DRAFT_NUMBER_ONE_TARGETS`. Stage 1 inventories ownership only. Later stages convert relevant draft targets into certification targets after baseline measurement.

## Status Legend

| Status | Meaning |
|---|---|
| NOT_APPLICABLE_WITH_REASON | Benchmark does not apply to this stage and the reason is documented. |
| BASELINE_MEASURED | Replayable baseline exists, but not yet final certification law. |
| CERTIFICATION_TARGET_PASSED | Numeric target passed with replayable evidence. |
| BLOCKED_WITH_OWNER_AND_NEXT_ACTION | Benchmark is not ready; owner and next action are documented. |

## Global Benchmark Ownership

| Benchmark Family | Draft Targets Present | Current Repo Evidence | Stage 1 Status | Owner | Next Action |
|---|---:|---|---|---|---|
| Stage 1 docs reconciliation | yes | this artifact set | CERTIFICATION_TARGET_PASSED | Stage 1 | Maintain links and final report proof. |
| Minimal benchmark result envelope | yes | `BenchmarkTargetPacket`, `BenchmarkResultPacket`, `BenchmarkResultRepo`, and `Ph1fStore` append-only target/result rows | CERTIFICATION_TARGET_PASSED | Stage 2A | Preserve the minimal envelope while future stages add product benchmark corpora/results. |
| Provider safety, provider-off, and early consent baseline | yes | `ph1providerctl.rs`, `runtime_bootstrap.rs`, `provider_secrets.rs`, `ph1kms.rs`, `ph1cost.rs`, `ph1quota.rs`, `ConsentStatePacket`, and `ConsentStateRepo` | CERTIFICATION_TARGET_PASSED | Stage 3A | Preserve zero-attempt/zero-dispatch provider-off proof and revocation-aware consent storage while Stage 4 consumes the baseline. |
| Activation/session/turn packet foundation | yes | `Stage4ActivationPacket`, `Stage4TurnBoundaryPacket`, `Stage4RecordBoundary`, `RuntimeCanonicalIngressRequest`, `SessionRuntimeProjection`, and `SessionTurnPermit` | CERTIFICATION_TARGET_PASSED | Stage 4A | Preserve no-route-authority, record-artifact separation, and packet alias crosswalk while Stage 5 consumes committed-turn boundaries. |
| Provider/model governance | yes | `ph1providerctl.rs`, `ph1kms.rs`, `ph1cost.rs`, `ph1quota.rs` | BLOCKED_WITH_OWNER_AND_NEXT_ACTION | Stage 3B, Stage 30 | Add prompt/model registries, champion router/model profile contracts, live-eval, fallback/rollback, and cost-quality target status. |
| Wake/activation | yes | `ph1w.rs`, wake migrations, native activation shells | BLOCKED_WITH_OWNER_AND_NEXT_ACTION | Stage 7 | Convert wake latency/false accept targets after baseline. |
| STT/listening | yes | `ph1k.rs`, `ph1c.rs`, `ph1listen.rs`, PH1.K telemetry fixtures | BLOCKED_WITH_OWNER_AND_NEXT_ACTION | Stage 8 | Build listening lab/gold corpus and measure WER/CER/latency. |
| Conversation/same-page | yes | session runtime and conversation concepts exist | BLOCKED_WITH_OWNER_AND_NEXT_ACTION | Stage 5, Stage 10 | Add continuity/open-loop/correction replay cases. |
| Scrambled language/meaning repair | yes | PH1.LANG/N/D/X/PRON/SRL exist | BLOCKED_WITH_OWNER_AND_NEXT_ACTION | Stage 10 | Create meaning repair benchmark corpus. |
| Math/science/history | yes | domain verification not complete as canonical expert lane | BLOCKED_WITH_OWNER_AND_NEXT_ACTION | Stage 11, Stage 13, Stage 14 | Add route/verifier benchmarks and replayable corpora. |
| Research/source quality | yes | `web_search_plan/eval`, `release`, `replay`, `trust`, `synthesis` | BLOCKED_WITH_OWNER_AND_NEXT_ACTION | Stage 13, Stage 14 | Convert citation/source targets into Stage 13/14 certification targets. |
| Search Operating System | yes | web-search sublanes and proof binaries exist | BLOCKED_WITH_OWNER_AND_NEXT_ACTION | Stage 13, Stage 14, Stage 24, Stage 34 | Tie public web, URL, cache, vertical, connector/API, deep research results into one benchmark envelope. |
| Write/display/TTS-safe split | yes | `ph1write.rs`, `ph1tts.rs`, adapter/native paths | BLOCKED_WITH_OWNER_AND_NEXT_ACTION | Stage 15, Stage 17 | Add source/debug/TTS leak and spoken/display benchmark cases. |
| TTS naturalness | yes | PH1.TTS exists | BLOCKED_WITH_OWNER_AND_NEXT_ACTION | Stage 17 | Add MOS/pronunciation/prosody target status and replay evidence. |
| Memory trust | yes | PH1.M, storage migration, memory DB tests | BLOCKED_WITH_OWNER_AND_NEXT_ACTION | Stage 21 | Add false-memory/stale/project-leak/forget proof benchmarks. |
| Human experience/emotion | yes | PH1.EMO.CORE, PH1.EMO.GUIDE, persona/learning modules | BLOCKED_WITH_OWNER_AND_NEXT_ACTION | Stage 29 | Add emotion boundary/user trust/frustration benchmark replay. |
| Multilingual | yes | PH1.LANG, PH1.PRON, STT/TTS surfaces | BLOCKED_WITH_OWNER_AND_NEXT_ACTION | Stage 32 | Add per-language, dialect, code-switch, protected non-English certification packs. |
| Native/runtime parity | yes | Mac/iPhone source, adapter transport, Android/Windows gaps | BLOCKED_WITH_OWNER_AND_NEXT_ACTION | Stage 33 | Build parity harness for existing surfaces and planned/missing reports. |
| Full certification | yes | historical proof ledgers exist but no canonical Stage 34 harness | BLOCKED_WITH_OWNER_AND_NEXT_ACTION | Stage 34 | Run all target families together after stages pass. |

## Stage Completion Benchmark Rule

No future stage may be marked `PROVEN_COMPLETE` unless every relevant benchmark family is one of:

- `NOT_APPLICABLE_WITH_REASON`
- `BASELINE_MEASURED`
- `CERTIFICATION_TARGET_PASSED`
- `BLOCKED_WITH_OWNER_AND_NEXT_ACTION`

For Stage 2A, the minimal benchmark envelope foundation is certified because the target/result packet, storage, idempotency, by-target lookup, latest-result lookup, and replay-safe comparison path now exist. For Stage 3A, the provider safety baseline is certified because provider-off zero-attempt/zero-dispatch proof, startup/health no-probe proof, and revocation-aware early consent storage now exist. All product quality benchmark families remain blocked by their owning future stage until their corpora and results are measured.
