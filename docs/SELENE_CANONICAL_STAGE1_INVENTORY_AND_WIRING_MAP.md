# Selene Canonical Stage 1 Inventory And Wiring Map

Status: PROVEN_COMPLETE_STAGE1_DOCS_RECONCILIATION
Build: CANONICAL_ENGINE_INVENTORY_AND_WIRING_MAP_REPAIR
Date: 2026-05-03
Repo Root: `/Users/selene/Documents/Selene-OS`
Scope: Docs and architecture reconciliation only.

## Startup Proof

The Stage 1 build started from a clean canonical repo state:

| Check | Result |
|---|---|
| `pwd` | `/Users/selene/Documents/Selene-OS` |
| `git status --short` | clean |
| `git status -sb` | `## main...origin/main` |
| branch | `main` |
| HEAD | `58d032471df6764faaa14e13037a73d56f0275a6` |
| origin/main | `58d032471df6764faaa14e13037a73d56f0275a6` |

## Canonical Plan Version Proof

`docs/SELENE_CANONICAL_MASTER_BUILD_PLAN.md` contains the required canonical 34-stage markers:

- `Stage 34 - Full System Certification Harness`
- `Dependency DAG And Build Slice Execution Rule`
- `Benchmark Target Status Legend`
- `Search Operating System And ChatGPT-Level Search Layer`
- `GLOBAL NUMBER-ONE DRAFT BENCHMARK TARGETS`

Stage 1 did not continue from the older 27-stage roadmap.

## Files Read

Required first-read files:

- `AGENTS.md`
- `docs/SELENE_CANONICAL_MASTER_BUILD_PLAN.md`
- `docs/CORE_ARCHITECTURE.md`
- `docs/SELENE_BUILD_EXECUTION_ORDER.md`
- `docs/SELENE_AUTHORITATIVE_ENGINE_INVENTORY.md`
- `docs/COVERAGE_MATRIX.md`
- `docs/MASTER_BUILD_COMPLETION_PLAN.md`
- `docs/MASTER_BUILD_COMPLETION_LEDGER.md`

Additional repo evidence surfaces inspected:

- `crates/selene_kernel_contracts/src`
- `crates/selene_engines/src`
- `crates/selene_os/src`
- `crates/selene_adapter/src`
- `crates/selene_storage/migrations`
- `crates/selene_storage/tests`
- `crates/selene_tools/src`
- `crates/selene_replay/src`
- `apple/mac_desktop`
- `apple/iphone`
- `docs/DB_WIRING`
- `docs/ECM`
- `docs/PHASE_PLANS`

## Stage 1 Output Artifacts

| Artifact | Purpose |
|---|---|
| `docs/SELENE_CANONICAL_STAGE1_INVENTORY_AND_WIRING_MAP.md` | Repo-truth inventory, wiring map, status summary, Stage 2 verdict. |
| `docs/SELENE_CANONICAL_DEPENDENCY_DAG.md` | Dependency graph for all 34 stages, packets, downstream consumers, blockers, and parallel notes. |
| `docs/SELENE_CANONICAL_GOLDEN_JOURNEY_MATRIX.md` | Initial golden journeys that later stages must protect. |
| `docs/SELENE_CANONICAL_STAGE_BUILD_SLICE_MAP.md` | Build-family slice map for broad stages. |
| `docs/SELENE_CANONICAL_BENCHMARK_TARGET_STATUS_MATRIX.md` | Benchmark target status and owner matrix. |

## Repo Surface Summary

| Area | Repo Evidence | Stage 1 Status | Canonical Owner |
|---|---|---|---|
| Rust crate spine | `selene_kernel_contracts`, `selene_engines`, `selene_os`, `selene_adapter`, `selene_storage`, `selene_tools`, `selene_replay` | EXISTS_BUT_NEEDS_RECONCILIATION | Stages 2-34 |
| Runtime foundations | `runtime_execution.rs`, `runtime_governance.rs`, `runtime_law.rs`, `runtime_bootstrap.rs`, `runtime_request_foundation.rs`, `runtime_session_foundation.rs`, `runtime_ingress_turn_foundation.rs`, `app_ingress.rs`, `section40_exit.rs` | EXISTS_BUT_NEEDS_RECONCILIATION | Stages 2, 4, 5, 12 |
| Storage/proof | migrations `0001_ph1f_foundation.sql` through `0025_wake_learn_signal_outbox.sql`, `selene_storage/src/ph1f.rs`, `selene_storage/src/ph1j.rs`, DB wiring tests | EXISTS_BUT_NEEDS_RECONCILIATION | Stage 2 |
| Provider/KMS/cost/quota | `provider_secrets.rs`, `device_vault.rs`, `vault_cli.rs`, `ph1providerctl.rs`, `ph1kms.rs`, `ph1cost.rs`, `ph1quota.rs` | EXISTS_BUT_NEEDS_RECONCILIATION | Stage 3 |
| Activation/session/turn | `app_ingress.rs`, `runtime_request_foundation.rs`, `runtime_session_foundation.rs`, `runtime_ingress_turn_foundation.rs` | PARTIALLY_BUILT | Stages 4-5 |
| Wake/listen/STT/audio | `ph1w.rs`, `ph1k.rs`, `ph1c.rs`, `ph1listen.rs`, `ph1endpoint.rs`, `ph1wake_training.rs`, wake/audio migrations | PARTIALLY_BUILT | Stages 7-8 |
| Voice ID | `ph1_voice_id.rs`, migration `0008_ph1vid_voice_enrollment_tables.sql`, voice DB wiring tests | PARTIALLY_BUILT | Stage 9 |
| Access/policy/tenant | `ph1access.rs`, `ph1policy.rs`, `ph1tenant.rs`, access migrations and DB wiring tests | EXISTS_BUT_NEEDS_RECONCILIATION | Stage 6 |
| Understanding/language | `ph1lang.rs`, `ph1srl.rs`, `ph1n.rs`, `ph1d.rs`, `ph1x.rs`, `ph1pron.rs`, `ph1prune.rs`, `ph1diag.rs`, `ph1explain.rs` | PARTIALLY_BUILT | Stage 10 |
| Routing/risk/protected closure | `ph1os.rs`, `ph1gov.rs`, `ph1work.rs`, `ph1lease.rs`, `ph1simfinder.rs`, `simulation_executor.rs`, `runtime_law.rs` | PARTIALLY_BUILT | Stages 11-12 |
| Public tools/search | `ph1e.rs`, `ph1search.rs`, `ph1prefetch.rs`, `crates/selene_os/src/web_search_plan` | NEEDS_FINISHING | Stages 13-14 |
| Write/TTS | `ph1write.rs`, `ph1tts.rs` in contracts, engines, OS; DB wiring docs/tests | PARTIALLY_BUILT | Stages 15, 17 |
| Presentation contracts | current native/adapter surfaces exist, but canonical rich presentation contract is not isolated as Stage 16 | NEEDS_BUILDING | Stage 16 |
| Adapter/protocol | `http_adapter.rs`, `grpc_adapter.rs`, `desktop_wake_life.rs`, `desktop_mic_producer.rs`, `desktop_capture_bundle_valid.rs`, `selene_adapter/src/lib.rs` | PARTIALLY_BUILT | Stage 18 |
| Mac Desktop | `apple/mac_desktop/SeleneMacDesktop` with `SeleneMacDesktopApp.swift`, `SeleneMacDesktopRuntimeBridge.swift`, `DesktopSessionShellView.swift` | PARTIALLY_BUILT | Stage 19 |
| Windows Desktop | no source tree found in repo evidence | NEEDS_BUILDING | Stage 19 |
| iPhone | `apple/iphone/SeleneIPhone` with `SeleneIPhoneApp.swift`, `SessionShellView.swift` | PARTIALLY_BUILT | Stage 20 |
| Android | planning docs under `docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE`; no Android source tree found | NEEDS_BUILDING | Stage 20 |
| Memory/persona/context | `ph1m.rs`, `ph1persona.rs`, `ph1context.rs`, `ph1kg.rs`, memory migrations and DB wiring tests | PARTIALLY_BUILT | Stage 21 |
| File/doc/data/vision | `ph1doc.rs`, `ph1vision.rs`, `ph1vision_media.rs`, web-search document and vision sublanes | PARTIALLY_BUILT | Stage 22 |
| Canvas/artifacts | `ph1art.rs`, `device_artifact_sync.rs`, artifacts ledger migration/tests; no full canonical canvas product surface found | NEEDS_BUILDING | Stage 23 |
| Apps/connectors/tasks | `ph1agent.rs`, `ph1sched.rs`, connector/app concepts in plan; live app directory/API connector contracts not fully surfaced | NEEDS_BUILDING | Stage 24 |
| Broadcast/delivery/reminders | `ph1bcast.rs`, `ph1delivery.rs`, `ph1rem.rs` | PARTIALLY_BUILT | Stage 25 |
| Business workflows | `ph1link.rs`, `ph1onb.rs`, `ph1position.rs`, `ph1capreq.rs` | PARTIALLY_BUILT | Stage 26 |
| Record mode | record boundaries appear in adapter/native history, but canonical full record product is partial | PARTIALLY_BUILT | Stage 27 |
| Image/video generation | generated-vs-sourced separation exists in search/vision context; video generation/editing product lane not found as full engine | NEEDS_BUILDING | Stage 28 |
| Learning/emotion/adaptation | `ph1feedback.rs`, `ph1learn.rs`, `ph1know.rs`, `ph1cache.rs`, `ph1pae.rs`, `ph1pattern.rs`, `ph1rll.rs`, `ph1emocore.rs`, `ph1emoguide.rs`, `ph1multi.rs` | PARTIALLY_BUILT | Stage 29 |
| Builder/replay/self-heal/dev | `ph1builder.rs`, `ph1selfheal.rs`, `selene_replay`, Section 07 reopen tools, web-search proof binaries | PARTIALLY_BUILT | Stage 30 |
| Privacy/admin/export/health | `ph1health.rs`, `ph1export.rs`, policy/tenant/KMS/consent-related carriers | PARTIALLY_BUILT | Stage 31 |
| Language certification | language engines exist; advanced certification packs not found as final canonical certification surface | NEEDS_BUILDING | Stage 32 |
| Parity/final certification | prior proof ledgers exist; canonical 34-stage parity/final harness not built | NEEDS_BUILDING | Stages 33-34 |

## Exact Repo Surface Audit Anchors

| Anchor Family | Repo Evidence | Status |
|---|---|---|
| `selene.rs` | `crates/selene_tools/src/bin/selene.rs` | EXISTS |
| `SeleneMacDesktopApp` | `apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopApp.swift` | EXISTS |
| `PH1_LEARN_FEEDBACK_KNOW` | `docs/DB_WIRING/PH1_LEARN_FEEDBACK_KNOW.md`, `docs/ECM/PH1_LEARN_FEEDBACK_KNOW.md`, `crates/selene_storage/tests/ph1_learn_feedback_know/db_wiring.rs` | EXISTS |
| storage/migration tokens | `ph1f`, `work_orders`, `ph1l`, `ph1vid`, `access_instance`, `ph1k`, `ph1link`, `access_master`, `access_ap`, `self_heal`, `ph1m`, `wake_artifact`, `wake_learn_signal` found in migrations/tests/docs | EXISTS |
| adapter/proof binaries | `http_adapter.rs`, `grpc_adapter.rs`, `desktop_wake_life.rs`, `desktop_mic_producer.rs`, `desktop_capture_bundle_valid.rs` | EXISTS |
| web-search proof binaries | `web_search_turn`, `web_search_enterprise_turn`, `web_search_eval_report`, `web_search_release_evidence`, `web_search_vision_turn` | EXISTS |
| Section 07 proof tools | `section07_reopen_detector.rs`, `section07_reopen_scan.rs` | EXISTS |

## Search Operating System Lane Inventory

| Lane | Repo Evidence | Status | Owner |
|---|---|---|---|
| Public web | `web_search_plan/web_provider`, `planning`, `parity`, `runtime`, `ph1e.rs` | PARTIAL | Stage 13 |
| News | `web_search_plan/news`, `gdelt`, `brave_news` | PARTIAL | Stage 13 |
| URL fetch/page reader | `web_search_plan/url`, `document/pdf_fetch.rs`, `document/pdf_text.rs`, `document/pdf_tables.rs` | PARTIAL | Stage 13, Stage 22 |
| Source verification | `web_search_plan/trust`, `synthesis/citation_validator.rs`, `synthesis/claim_extractor.rs` | PARTIAL | Stage 13 |
| Claim/citation system | `web_search_plan/chunk`, `synthesis`, `write/citation_renderer.rs` | PARTIAL | Stages 13-16 |
| Deep research | `web_search_plan/multihop`, `enterprise`, `release`, `replay`, `eval` | PARTIAL | Stage 14 |
| Cache/offline | `web_search_plan/cache`, `replay`, `temporal` | PARTIAL | Stage 13 |
| Realtime verticals | `web_search_plan/realtime/adapters/weather.rs`, `finance.rs`, `flights.rs`, `generic_json.rs` | PARTIAL | Stage 13 |
| Image evidence | `web_search_plan/vision`, `ph1vision.rs`, image packet history in ledgers | PARTIAL | Stage 13, Stage 28 |
| Shopping/product | `web_search_plan/structured/adapters/pricing_products.rs`, `competitive/pricing_normalize.rs` | PARTIAL | Stage 13 |
| Academic/government/filings | `web_search_plan/structured/adapters/academic.rs`, `gov_dataset.rs`, `filings.rs`, `company_registry.rs`, `document/filing` | PARTIAL | Stage 13 |
| Connector route | canonical plan owns route; live app/connector search surface not fully found | NEEDS_BUILDING | Stage 24 |
| Platform API route | canonical plan owns route; specific API capability registry not fully found | NEEDS_BUILDING | Stage 24 |
| Agent browser route | canonical plan owns route; visual browser/watch implementation not found | NEEDS_BUILDING | Stage 24 |
| Search-to-Write | `web_search_plan/write`, `ph1write.rs`, adapter/native presentation history | PARTIAL | Stages 15-16 |
| Search certification | `web_search_eval_report`, `web_search_release_evidence`, `web_search_plan/eval`, `release`, `replay` | PARTIAL | Stages 14, 30, 34 |

## Global Benchmark And Lab Lane Inventory

| Lane | Repo Evidence | Status | Owner |
|---|---|---|---|
| Global benchmark target table | `docs/SELENE_CANONICAL_MASTER_BUILD_PLAN.md` | DRAFT_TARGET_PRESENT | Stages 1-34 |
| Benchmark result envelope | not found as canonical global packet/storage surface | NEEDS_BUILDING | Stage 2 |
| Competitor parity | web-search competitive modules exist; global competitor parity report package not found | PARTIAL | Stage 30 |
| Provider championship router | provider control exists; model champion contracts not found as complete router | NEEDS_BUILDING | Stage 3, Stage 30 |
| STT/listening lab | PH1.K/PH1.C/PH1.LISTEN exist; world-class lab/corpus ownership is not final | PARTIAL | Stage 8 |
| TTS leaderboard | PH1.TTS exists; MOS/pronunciation benchmark lane not final | PARTIAL | Stage 17, Stage 34 |
| Research leaderboard | web-search eval/release exists; number-one research targets not final | PARTIAL | Stages 13-14, 30, 34 |
| Memory leaderboard | PH1.M exists; false-memory/stale/project leakage target lane not final | PARTIAL | Stage 21, Stage 34 |
| Human experience lab | PH1.EMO/PH1.PERSONA exists; user-trust/companion benchmark lane not final | PARTIAL | Stage 29, Stage 34 |
| Multilingual leaderboard | PH1.LANG exists; certification packs not final | PARTIAL | Stage 32 |

## ChatGPT-Parity Product Lane Inventory

| Product Lane | Repo Evidence | Status | Owner |
|---|---|---|---|
| Custom assistants / GPT-like builder | `ph1builder.rs`, builder tables, plan lane only for assistant store/actions/knowledge | PARTIAL | Stage 30 |
| App directory / SDK / MCP apps | `ph1agent.rs` exists; app directory/SDK/MCP contracts not complete | NEEDS_BUILDING | Stage 24 |
| Interactive app cards | presentation/native transport not yet canonicalized | NEEDS_BUILDING | Stage 16, Stage 24 |
| Visual agent browser/watch mode | not found as complete source surface | NEEDS_BUILDING | Stage 24 |
| Visible data analysis/charts | document/data/vision carriers exist; canonical data sandbox/chart product not complete | NEEDS_BUILDING | Stage 22 |
| Canvas share/export/version restore | artifact sync exists; canonical canvas product not complete | NEEDS_BUILDING | Stage 23 |
| Study/tutor mode | learning/persona exists; study/tutor product mode not complete | NEEDS_BUILDING | Stage 29 |
| Shopping/product cards | structured pricing/product/search evidence partial | PARTIAL | Stage 13, Stage 16 |
| Video generation/editing | vision/video internals exist; generated video product lane not complete | NEEDS_BUILDING | Stage 28 |

## Canonical Packet And Handoff Inventory

| Canonical Packet / Envelope | Repo Equivalent Evidence | Stage 1 Status | Owner |
|---|---|---|---|
| `ActivationPacket` | `app_ingress.rs`, adapter request paths, native activation clients | PARTIAL | Stage 4 |
| `ConsentStatePacket` | consent concepts in master plan, wake/Voice ID/storage policy surfaces | NEEDS_RECONCILIATION | Stage 3 |
| `DeviceTrustPacket` | policy/device trust concepts, runtime execution envelope fields | NEEDS_RECONCILIATION | Stage 3 |
| `ProviderBudgetPacket` | `ph1providerctl.rs`, `ph1cost.rs`, `ph1quota.rs` | PARTIAL | Stage 3 |
| `SessionPacket` | `runtime_session_foundation.rs`, `ph1l.rs` | EXISTS_BUT_NEEDS_RECONCILIATION | Stages 4-5 |
| `TurnCandidatePacket` | `runtime_ingress_turn_foundation.rs`, `app_ingress.rs`, adapter voice turn request | PARTIAL | Stage 4 |
| `CommittedTurnPacket` | `runtime_ingress_turn_foundation.rs` | PARTIAL | Stage 5 |
| `RecordSessionPacket`, `AudioArtifactPacket` | record fields in master plan, artifact ledger/device sync surfaces | NEEDS_RECONCILIATION | Stage 27 |
| `VoiceIdentityPacket` | `ph1_voice_id.rs`, runtime execution voice identity state | PARTIAL | Stage 9 |
| `AccessContextPacket` | `ph1access.rs`, `runtime_governance.rs`, `runtime_law.rs` | PARTIAL | Stage 6 |
| `UnderstandingPacket` | `ph1lang.rs`, `ph1n.rs`, `ph1d.rs`, `ph1x.rs` | PARTIAL | Stage 10 |
| `RouteCandidatePacket` | routing concepts in `ph1x.rs`/`app_ingress.rs`; canonical Stage 11 packet needs closure | NEEDS_BUILDING | Stage 11 |
| `RiskDecisionPacket`, `AuthorityDecisionPacket`, `SimulationResultPacket` | `runtime_governance.rs`, `runtime_law.rs`, `simulation_executor.rs` | PARTIAL | Stage 12 |
| `ExecutionApprovalPacket`, `FailClosedResponsePacket` | `runtime_execution.rs`, `runtime_law.rs`, adapter fail-closed outcomes | PARTIAL | Stage 12 |
| `EvidencePacket`, `ImageEvidencePacket` | `ph1e.rs`, `web_search_plan`, `ph1vision.rs`, adapter source/image packet history | PARTIAL | Stage 13 |
| Search packets | `web_search_plan` sublanes, `ph1search.rs`, `ph1e.rs` | PARTIAL | Stages 13-14 |
| `WriteResponsePacket` | `ph1write.rs` | PARTIAL | Stage 15 |
| `PresentationEnvelope` | not isolated as canonical contract yet | NEEDS_BUILDING | Stage 16 |
| `AdapterResponsePacket`, `ClientRenderPacket` | `selene_adapter/src/lib.rs`, native shells | PARTIAL | Stage 18 |
| `TtsPacket` | `ph1tts.rs`, adapter/native TTS paths | PARTIAL | Stage 17 |
| Memory/persona/emotion/prosody packets | `ph1m.rs`, `ph1persona.rs`, `ph1emocore.rs`, `ph1emoguide.rs`, `ph1tts.rs` | PARTIAL | Stages 21, 29 |
| Benchmark/model/listening/research/domain packets | mostly canonical plan concepts; web-search eval partial | NEEDS_BUILDING | Stages 2, 3, 8, 13, 14, 30, 34 |
| App/data/canvas/study/shopping/video packets | partial artifact/search/data evidence; product packets not complete | NEEDS_BUILDING | Stages 16, 22-24, 28-30 |
| `AuditTracePacket` | `ph1j.rs`, audit/proof ledgers, runtime execution envelope | PARTIAL | Stage 2 |

## High-Level Runtime Wiring Map

Canonical target wiring and current repo evidence align in this order:

```text
native/adapter activation
-> app_ingress / runtime_request_foundation
-> runtime_session_foundation / ph1l
-> runtime_ingress_turn_foundation
-> ph1k/ph1c/ph1listen or typed/record boundary
-> ph1_voice_id and access/policy/tenant context
-> ph1lang/ph1n/ph1d/ph1x understanding
-> Stage 11 route candidate ownership
-> runtime_governance / simulation_executor / runtime_law
-> ph1e/search/doc/vision or protected fail-closed/approval
-> ph1write
-> Stage 16 presentation contracts
-> selene_adapter HTTP/gRPC/native transport
-> Mac/iPhone now, Android/Windows later
-> ph1tts playback control where approved
-> ph1j/audit/replay/release evidence
```

Current risk: the repo has many carriers and proof surfaces, but several canonical packet names are aliases or future contracts rather than one-to-one source symbols. Later stages must reconcile repo-truth names instead of inventing duplicate brains.

## Duplicate, Legacy, Or Drift Findings

| Finding | Evidence | Required Handling |
|---|---|---|
| Older Section 01-11 and H-build truth remains active historical evidence. | `docs/SELENE_BUILD_EXECUTION_ORDER.md`, `docs/COVERAGE_MATRIX.md`, `docs/MASTER_BUILD_COMPLETION_PLAN.md`, `docs/MASTER_BUILD_COMPLETION_LEDGER.md` | Do not delete. Map into 34-stage owners during Stage 2+ reconciliation. |
| `MASTER_BUILD_COMPLETION_PLAN.md` is still organized around Build Sections 01-11 plus app phases. | section summary rows in `docs/MASTER_BUILD_COMPLETION_PLAN.md` | Treat as evidence ledger, not replacement roadmap. |
| Some canonical packet names are roadmap-level names, while repo symbols use PH1/H/Section names. | packet inventory above | Stage 2 must define alias/crosswalk rather than duplicate source files. |
| Android is planning-only in current repo evidence. | `docs/PHASE_PLANS/PHASE_B_ANDROID_WAKE` and no Android source tree found | Stage 20 must record planned/missing until source tree exists. |
| Windows Desktop source tree was not found. | no Windows source path found in repo scan | Stage 19 must record planned/missing until source tree exists. |
| Stage 13 is too broad as one implementation build. | canonical plan and web-search leaf volume | Use `docs/SELENE_CANONICAL_STAGE_BUILD_SLICE_MAP.md`; do not run Stage 13 as one build. |

No roadmap contradiction requiring JD approval was found. The main reconciliation note is that older Section/H truth must remain evidence while the 34-stage plan becomes the canonical execution roadmap.

## Stage 2 Verdict

Stage 2 is ready to start with a narrowed reconciliation scope:

```text
Stage 2A - Runtime Kernel, Storage, Proof Ledger, Law Foundation, And Minimal Benchmark Envelope Inventory Reconciliation
```

Stage 2 should not rebuild runtime/storage/law from zero. It should reconcile and prove the existing runtime execution envelope, request/session/turn foundations, PH1.F/PH1.J storage, law/governance carriers, trace fields, idempotency, append-only audit, and the new minimal benchmark target/result envelope required by the number-one quality system.

No Stage 2 blocker was found. The must-not-start-early rule is that Stage 3 provider/model governance and all later feature stages must wait until Stage 2 has the minimal proof/replay and benchmark envelope crosswalk.

