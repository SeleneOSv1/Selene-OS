# Stage 7.1 Durable Internal History Evidence Replay

## 1. Executive Conclusion

Stage 7.1 passes.

Stage 7 internal-history evidence is no longer process-memory-only for the live inspection path. Evidence filed by the real Desktop app is synchronized into the adapter persistence journal, restored into PH1.F storage on adapter bootstrap, and replayed through `/v1/ui/internal-history/evidence` after adapter restart.

Readiness:

`READY_FOR_STAGE_8_FRESH_MEMORY`

## 2. Root Cause Found

Two owner-local issues were found.

First, Stage 7 evidence was appended into the live PH1.F store and visible while the adapter process remained alive, but the adapter endpoint did not restore/replay the PH1.F evidence records from durable persistence after restart. The endpoint therefore behaved like a live in-process report even though the evidence records existed as canonical PH1.F contracts.

Second, real Desktop cough/noise rejection after wake was visually correct but was not being filed. Desktop scheduled the rejected-evidence transport, but the adapter returned a contract violation because the PH1.C audit idempotency key became too long after PH1.C appended its own suffix. The Swift bridge previously swallowed the failure, so the UI behavior looked correct while the ledger evidence was missing.

Owners:

- Storage / PH1.F: durable evidence record restore and replay.
- Adapter: persistence bridge, evidence endpoint bridge, rejected voice evidence route, and bounded idempotency key construction.
- Desktop transport: capture-surface rejected-evidence transport only. Desktop still does not decide memory, session meaning, or runtime semantics.

## 3. Files Changed

- `crates/selene_kernel_contracts/src/common.rs`
- `crates/selene_kernel_contracts/src/ph1j.rs`
- `crates/selene_kernel_contracts/src/ph1l.rs`
- `crates/selene_kernel_contracts/src/ph1_voice_id.rs`
- `crates/selene_kernel_contracts/src/ph1f.rs`
- `crates/selene_storage/src/ph1f.rs`
- `crates/selene_adapter/src/lib.rs`
- `crates/selene_adapter/src/bin/http_adapter.rs`
- `apple/mac_desktop/SeleneMacDesktop/DesktopSessionShellView.swift`
- `apple/mac_desktop/SeleneMacDesktop/SeleneMacDesktopRuntimeBridge.swift`
- `docs/reports/STAGE_7_1_DURABLE_INTERNAL_HISTORY_EVIDENCE_REPLAY.md`

No Desktop semantic owner was added. Desktop only transports rejected capture evidence already decided by capture/transcript conditions.

## 4. Storage / Schema Decision

No new migration was required.

The existing Stage 7 PH1.F evidence contract and Stage 7 storage shape were sufficient. Stage 7.1 wires durable append/replay by:

- adding PH1.F record restore APIs in `crates/selene_storage/src/ph1f.rs`;
- persisting `InternalHistoryEvidenceRecord` entries in adapter persistence state;
- restoring persisted records into PH1.F on adapter bootstrap;
- syncing the live PH1.F store back to persistence before endpoint reports;
- serving the redacted evidence endpoint from persisted durable records when available.

No parallel ledger, JSON temp ledger, Desktop-owned archive, or second memory store was created.

## 5. Pre-Restart Live Evidence Proof

Latest app proof before live smoke:

- Repo HEAD: `ad2eb4796de4b12bb445c2a6e923f1378d9aa8bc`
- Bundle path: `/tmp/selene_stage7_1_durable_evidence_ad2eb47/Build/Products/Debug/SeleneMacDesktop.app`
- App PID before restart: `36517`
- Adapter PID before restart: `36561`
- One app instance and one adapter listener were present.
- `/healthz` reported the current repo head and the same bundle path.

JD live test:

- JD woke Selene and coughed.
- Visible result: no user bubble, no Selene answer, no TTS.
- Transcript endpoint after the cough: `message_count = 0`.

Pre-restart evidence captured to `/tmp/selene_stage7_1_pre_restart_evidence.json`:

| Evidence ID | Kind | Correlation ID | Turn ID | Modality | PH1.C Status | Memory Candidate |
| --- | --- | --- | --- | --- | --- | --- |
| 45 | RejectedInput | `1779001755768000000` | `1779001755768000000` | Voice | Rejected | BlockedRejectedTranscript |
| 46 | LifecycleBoundary | `1779001755768000000` | `1779001755768000000` | System | NotApplicable | NotApplicable |
| 47 | RejectedInput | `1779001779218000000` | `1779001779218000000` | Voice | Rejected | BlockedRejectedTranscript |
| 48 | LifecycleBoundary | `1779001779218000000` | `1779001779218000000` | System | NotApplicable | NotApplicable |

The first rejected pair records the empty-transcript cough/noise rejection. The second pair records the Stage 6 idle-close active capture with no committed transcript.

## 6. Adapter Restart Proof

After the pre-restart evidence was captured, stale app and adapter instances were closed and the same latest bundle was relaunched.

Post-restart provenance:

- Repo HEAD from `/healthz`: `ad2eb4796de4b12bb445c2a6e923f1378d9aa8bc`
- Bundle path from `/healthz`: `/tmp/selene_stage7_1_durable_evidence_ad2eb47/Build/Products/Debug/SeleneMacDesktop.app`
- App PID after restart: `37165`
- Adapter PID after restart: `37213`
- Adapter bind: `127.0.0.1:18765`
- Managed by: `SeleneMacDesktopRuntimeBridge`
- `pgrep -fl SeleneMacDesktop` returned one Desktop app.
- `lsof -nP -iTCP:18765 -sTCP:LISTEN` returned one adapter listener.

## 7. Post-Restart Durable Evidence Proof

After adapter restart, `/v1/ui/internal-history/evidence` returned the same pre-restart rows:

| Evidence ID | Kind | Correlation ID | Turn ID | Modality | PH1.C Status | Memory Candidate |
| --- | --- | --- | --- | --- | --- | --- |
| 45 | RejectedInput | `1779001755768000000` | `1779001755768000000` | Voice | Rejected | BlockedRejectedTranscript |
| 46 | LifecycleBoundary | `1779001755768000000` | `1779001755768000000` | System | NotApplicable | NotApplicable |
| 47 | RejectedInput | `1779001779218000000` | `1779001779218000000` | Voice | Rejected | BlockedRejectedTranscript |
| 48 | LifecycleBoundary | `1779001779218000000` | `1779001779218000000` | System | NotApplicable | NotApplicable |

JD then woke Selene and coughed again after the restart.

Visible result:

- no committed user message;
- no Selene reply;
- no TTS;
- transcript endpoint remained `message_count = 0`;
- Selene later slept by the 30-second idle-close rule.

Post-restart new evidence:

| Evidence ID | Kind | Correlation ID | Turn ID | Modality | PH1.C Status | Memory Candidate |
| --- | --- | --- | --- | --- | --- | --- |
| 50 | RejectedInput | `1779001903782000000` | `1779001903782000000` | Voice | Rejected | BlockedRejectedTranscript |
| 51 | LifecycleBoundary | `1779001903782000000` | `1779001903782000000` | System | NotApplicable | NotApplicable |
| 52 | RejectedInput | `1779001929993000000` | `1779001929993000000` | Voice | Rejected | BlockedRejectedTranscript |
| 53 | LifecycleBoundary | `1779001929993000000` | `1779001929993000000` | System | NotApplicable | NotApplicable |

The endpoint total after restart was `53` evidence events.

## 8. Exact Evidence IDs Verified

- Pre-restart cough/empty transcript: `internal_history_event_id = 45`, `correlation_id = 1779001755768000000`.
- Pre-restart cough note: `internal_history_event_id = 46`.
- Pre-restart idle close no committed transcript: `internal_history_event_id = 47`, `correlation_id = 1779001779218000000`.
- Pre-restart idle close note: `internal_history_event_id = 48`.
- Post-restart cough/empty transcript: `internal_history_event_id = 50`, `correlation_id = 1779001903782000000`.
- Post-restart cough note: `internal_history_event_id = 51`.
- Post-restart idle close no committed transcript: `internal_history_event_id = 52`, `correlation_id = 1779001929993000000`.
- Post-restart idle close note: `internal_history_event_id = 53`.

All rejected voice evidence rows have:

- `event_kind = RejectedInput`
- `modality = Voice`
- `input.ph1c_status = Rejected`
- `ph1m.memory_candidate_status = BlockedRejectedTranscript`

## 9. JD Live Visible / Audible Results

JD confirmed:

- Selene answered normal spoken prompts earlier in Stage 7 live proof.
- Typed prompt behavior worked.
- Protected payroll request failed closed.
- The cough after wake produced no user bubble and no Selene reply.
- After the cough/noise test, Selene went to sleep normally by the 30-second idle close.

The repeated cough reports from JD were not treated as new failures. They were the same intended rejected-noise scenario and were verified against the ledger.

## 10. Controlled Reload / Replay Test Proof

Automated proof added:

- `crates/selene_storage/src/ph1f.rs`
  - `ph1f::tests::at_f_stage7_1_internal_history_evidence_restores_after_store_reload`
  - proves PH1.F evidence records can be restored into a fresh store, read back, and retain append-only order.

- `crates/selene_adapter/src/lib.rs`
  - `tests::stage7_1_durable_evidence_report_survives_adapter_restart`
  - files committed voice/time evidence, typed evidence, protected fail-closed evidence, rejected voice/noise evidence, tool/time evidence, and TTS evidence;
  - restarts/recreates the adapter runtime;
  - proves the redacted evidence report still returns the same evidence after restart;
  - proves rejected voice evidence remains `BlockedRejectedTranscript` and does not become a committed user turn.

## 11. Evidence Category Proof

Voice / spoken output:

- Controlled adapter test files voice evidence and TTS ready evidence and confirms both survive runtime restart.
- Live Desktop endpoint retains voice rejected-evidence rows after restart.

Typed:

- Controlled adapter test files a typed short-story turn.
- Typed actor identity is kept separate from Voice ID and does not fabricate voice identity evidence.

Protected:

- Controlled adapter test files a protected payroll fail-closed row.
- Evidence survives runtime restart and is not represented as completed execution.

Noise / cough / self-echo class:

- Live Desktop cough after wake creates no transcript message and no committed user turn.
- Rejected input evidence is filed with PH1.C status `Rejected`.
- PH1.M candidate status is `BlockedRejectedTranscript`.

Tool / time:

- Controlled adapter test files `time` tool-family evidence for time prompts and verifies restart replay.

Speaker / Voice ID:

- Speaker evidence remains nullable/evidence-only in PH1.F evidence records.
- Typed turns do not fabricate Voice ID fields.
- Voice ID is not used as protected authority.

PH1.X / PH1.M:

- PH1.X and PH1.M evidence slots remain present/nullable in the Stage 7 evidence shape.
- Rejected evidence never becomes a memory candidate.

## 12. Tests Run

Commands passed:

- `cargo fmt --all --check`
- `cargo check`
- `cargo build -p selene_adapter --bin selene_adapter_http`
- `cargo test -p selene_adapter stage7_1_durable_evidence_report_survives_adapter_restart -- --test-threads=1`
- `cargo test -p selene_kernel_contracts -- --test-threads=1`
- `cargo test -p selene_storage -- --test-threads=1`
- `cargo test -p selene_adapter -- --test-threads=1`
- `cargo test -p selene_os -- --test-threads=1`
- `cargo test -p selene_engines -- --test-threads=1`
- `xcodebuild -project apple/mac_desktop/SeleneMacDesktop.xcodeproj -scheme SeleneMacDesktop -configuration Debug -derivedDataPath /tmp/selene_stage7_1_durable_evidence_ad2eb47 build`
- `git diff --check`

Notable full-suite results:

- `selene_kernel_contracts`: 395 passed.
- `selene_storage`: all unit, DB wiring, and doctests passed.
- `selene_adapter`: 464 unit tests passed, 4 ignored by existing live-secret gating; binary and capture tests passed.
- `selene_os`: 1,682 unit tests passed; binary tests passed.
- `selene_engines`: 662 unit tests passed, 12 ignored by existing live-secret gating; doctests passed.
- Xcode build succeeded.

## 13. Carried-Forward Proof

Carried-forward behavior remains intact:

- Latest Desktop bundle was used and proved through `/healthz`.
- One app instance and one managed adapter listener were proved.
- Wake accepted after restart.
- 30-second idle close still occurred after the cough/noise path.
- Rejected cough/noise did not create a committed user message.
- OpenAI STT was not shown to remain active as a committed turn after rejection.
- TTS evidence remains filed in the controlled restart proof for spoken output.
- Typed evidence remains filed in the controlled restart proof.
- Protected fail-closed evidence remains filed in the controlled restart proof.
- Desktop clean transcript remained empty for rejected cough.

## 14. Final Readiness

Stage 7.1 is accepted.

The internal-history evidence ledger is durable enough for Stage 8 fresh memory work:

- evidence is appended;
- evidence is persisted;
- evidence is restored after adapter restart;
- evidence is replayed through the approved endpoint;
- rejected voice/noise evidence is filed without becoming memory;
- committed voice, typed, protected, tool/time, and TTS evidence survive controlled restart proof.

Readiness:

`READY_FOR_STAGE_8_FRESH_MEMORY`
