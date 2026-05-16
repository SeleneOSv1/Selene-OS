# Stage 7 Speaker-Aware Immutable History Ledger

## Executive Conclusion

Stage 7 is implemented as an owner-local PH1.F Storage/archive/audit foundation. It adds an append-only internal-history evidence ledger that records compact evidence refs around committed and rejected interactions. The build does not add user-facing archive/search UI, does not make Desktop a memory owner, and does not implement PH1.M recall behavior.

Readiness line: `STAGE_7_LEDGER_FOUNDATION_READY_FOR_STAGE_8_FRESH_MEMORY`.

## Owner Map

- Storage/archive/audit owner: `crates/selene_kernel_contracts/src/ph1f.rs`, `crates/selene_storage/src/ph1f.rs`, `crates/selene_storage/src/repo.rs`.
- Adapter committed-turn bridge owner: `crates/selene_adapter/src/lib.rs`.
- PH1.X evidence refs consumed by Stage 7: `ActiveContextPacket`, `HumanConversationDirective`.
- PH1.M evidence refs consumed by Stage 7: `MemoryEvidencePacket`, `MemoryRecallRequest`, `FreshMemoryHandoff`, `MemoryContinuationDecision`.
- PH1.VOICE.ID speaker evidence remains evidence-only and nullable.
- Desktop was not touched.

## What Changed

- Added PH1.F canonical Stage 7 internal-history contracts:
  - `InternalHistoryEventId`
  - `InternalHistoryEventKind`
  - `InternalHistoryModality`
  - `SpeakerEvidenceRefs`
  - `InputTranscriptEvidenceRefs`
  - `ResponseSpokenEvidenceRefs`
  - `LiveContextEvidenceRefs`
  - `MemoryEvidenceRefs`
  - `InternalHistoryEvidenceRefs`
  - `InternalHistoryEvidenceInput`
  - `InternalHistoryEvidenceRecord`
- Added in-memory PH1.F storage for `internal_history_evidence_ledger`.
- Added repository methods for appending and reading internal-history evidence rows.
- Added DB migration `0026_stage7_internal_history_evidence.sql`.
- Wired `append_conversation_turn` so every new committed conversation turn automatically appends a Stage 7 evidence row.
- Wired PH1.C rejected transcript commit so rejected voice evidence is stored while blocked from memory candidacy.
- Repaired the adapter committed-turn bridge so final runtime answer text is filed for committed user-final turns when no narrower final transcript text is already supplied. Partial-only transcript previews remain previews.

## Evidence Coverage

The ledger supports nullable refs for:

- speaker identity and Voice ID evidence
- typed actor identity evidence
- PH1.C transcript status and rejected-transcript reasons
- Selene final response text hashes and TTS/playback refs
- PH1.X active context and human conversation directive refs
- PH1.M memory evidence, recall request, fresh handoff, and continuation decision refs
- PH1.E tool/provider/source refs
- PH1.WRITE presentation refs
- image/file/screenshot/multimodal refs
- correction refs
- decision/task/project evidence candidates
- privacy/retention/trust refs
- protected fail-closed/no-execution refs
- timing/latency refs
- device/surface/provenance refs
- audit/replay/integrity refs

## Boundaries Preserved

- Stage 7 stores evidence refs, not full memory behavior.
- Raw audio, raw full webpages, raw images, and private files are not newly hoarded.
- Typed turns do not fabricate Voice ID evidence.
- Rejected transcript/noise/self-echo evidence cannot become a memory candidate.
- Protected fail-closed evidence is stored as blocked/no-execution evidence, not action success.
- PH1.X remains live-context owner.
- PH1.M remains memory owner.
- Adapter transports/bridges committed turns; it is not a memory brain.
- Desktop renders only and was not changed.

## Tests Added

- Kernel contract tests prove typed actor evidence, nullable voice speaker evidence, rejected transcript memory blocking, and PH1.X/PH1.M/tool/presentation/multimodal refs.
- Storage tests prove committed turns auto-file evidence, typed turns do not fabricate voice identity, rejected transcripts file blocked evidence, and protected fail-closed rows are append-only/idempotent.
- DB wiring test proves the Stage 7 migration exposes speaker, transcript, PH1.X, PH1.M, multimodal, protected, and replay-integrity columns.
- Adapter test proves the committed-turn bridge files internal-history evidence for both user input and Selene response text.

## Validation Summary

- `cargo check`
- `cargo test -p selene_kernel_contracts -- --test-threads=1`
- `cargo test -p selene_storage -- --test-threads=1`
- `cargo test -p selene_adapter -- --test-threads=1`
- `cargo test -p selene_os -- --test-threads=1`
- `cargo test -p selene_engines -- --test-threads=1`
- `git diff --check`

## Deferred

- Full PH1.M fresh/day/topic/deep recall behavior.
- Natural memory UI.
- PH1.WRITE presentation upgrade.
- Full image/file/web research memory.
- Live Voice ID authority. Voice ID remains evidence only.
