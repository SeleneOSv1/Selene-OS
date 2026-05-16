# Stage 6.6 PH1.X / PH1.M Canonical Contract Foundation

## Executive Summary

Stage 6.6 adds the missing canonical PH1.X and PH1.M contract layer that the Stage 6.5 repo-truth audit required before Stage 7.

Readiness line:

READY_FOR_STAGE_7_CONTRACTS

This build is contract-only. It does not implement Stage 7 immutable storage, does not rewrite PH1.X or PH1.M behavior, does not change Desktop, and does not expand storage schema.

## Owner Files Changed

- `crates/selene_kernel_contracts/src/ph1x.rs`
- `crates/selene_kernel_contracts/src/ph1m.rs`
- `docs/reports/STAGE_6_6_PH1X_PH1M_CONTRACT_FOUNDATION.md`

## Owner Files Not Touched

- `crates/selene_engines/src/ph1x.rs`
- `crates/selene_engines/src/ph1m.rs`
- `crates/selene_adapter/src/lib.rs`
- `crates/selene_os/src/app_ingress.rs`
- `crates/selene_storage/**`
- `apple/mac_desktop/**`

## Contracts Added Or Formalized

### PH1.X

`ActiveContextPacket` was added as the canonical PH1.X live-attention packet.

It carries:

- active topic and intent
- interaction posture
- conversation rhythm
- continuation type
- reference target
- entity focus
- tool family
- writing artifact
- pending slots
- correction target
- topic stack
- response shape
- confidence
- ambiguity level
- protected risk
- memory handoff flag
- suggested next engine
- evidence refs

`HumanConversationDirective` was added as the canonical PH1.X next-step directive evidence contract.

It supports:

- continue current topic
- modify previous output
- correct previous output
- answer new question
- ask clarification
- hand off to PH1.M
- route to PH1.E/tool
- route to PH1.WRITE
- fail closed for protected action
- wait/no action

Compatibility:

`impl From<&Ph1xDirective> for HumanConversationDirective` maps the existing PH1.X executable directive into the new evidence directive without replacing active behavior.

### PH1.M

`MemoryEvidencePacket` was added as the canonical remembered-context evidence packet.

It carries:

- memory type
- topic label
- age label
- confidence
- evidence refs
- continuation allowed
- clarification needed
- user-facing summary
- active-context allowed
- user-facing recall style
- trust level
- privacy status
- conflict/staleness status

`MemoryRecallRequest` was added as a canonical compatibility wrapper around existing `Ph1mRecallRequest`.

Compatibility:

`Ph1mRecallRequest` remains the active PH1.M recall request owner. `MemoryRecallRequest` wraps it with Stage 7-ready context/ref fields rather than creating a duplicate recall engine.

`FreshMemoryHandoff` was added as the canonical PH1.L / PH1.X / PH1.M bridge evidence contract for sleep/wake continuation.

It carries:

- handoff id
- source session/thread/turn refs
- last topic
- last intent
- last tool family
- last entity focus
- last answer type
- freshness label
- confidence
- evidence refs
- continuation allowed
- handoff reason
- expiry/decay timestamp when available

`MemoryContinuationDecision` was added as the canonical PH1.M answer to whether a remembered topic should continue.

It supports:

- continue automatically
- ask clarification
- answer normally
- no memory match
- blocked by privacy
- blocked by staleness
- blocked by low confidence

## Compatibility Choices

- Existing `ThreadState`, `LastTurnContext`, and `Ph1xDirective` remain active compatibility paths.
- Existing `Ph1mRecallRequest`, recent archive recall, thread digest, and resume-select contracts remain active compatibility paths.
- Adapter-local context shortcuts remain active compatibility because this build does not wire behavior replacement.
- No old active path was removed because this build only creates canonical contracts; behavior replacement belongs to later PH1.X/PH1.M builds.

## Old Or Conflicting Paths

No conflicting canonical type names existed before this build.

Retained active compatibility paths:

- `Ph1xDirective` for current PH1.X execution.
- `ThreadState` and `LastTurnContext` for current active-thread state.
- `Ph1mRecallRequest` for current PH1.M recall.
- Adapter deterministic active context and recent archive recall shortcuts.

Future cleanup:

After Stage 7 stores canonical refs and later PH1.X/PH1.M builds wire behavior through these contracts, retained shortcut paths should be retired under the Clean Replacement / No Dead Legacy Path Law.

## Storage And Schema

Storage/schema expansion is deferred.

Stage 7 can now store immutable refs to:

- `ActiveContextPacket`
- `HumanConversationDirective`
- `MemoryEvidencePacket`
- `MemoryRecallRequest`
- `FreshMemoryHandoff`
- `MemoryContinuationDecision`

No storage schema was changed in this build.

## Stage 7 Evidence Refs Now Available

Stage 7 can preserve/store refs for:

- PH1.X active topic/intent/posture/rhythm/continuation data
- PH1.X reference target, entity focus, tool family, writing artifact, pending slots, corrections, topic stack
- PH1.X response shape, ambiguity level, protected risk, memory handoff flag, suggested next engine
- PH1.X human conversation directive
- PH1.M memory evidence type, age label, trust, privacy, conflict/staleness state
- PH1.M recall request wrapper refs around existing `Ph1mRecallRequest`
- PH1.M fresh memory handoff refs after sleep/wake/session boundary
- PH1.M continuation decision refs for continue/clarify/normal/no-match/blocked outcomes

## Test Summary

Added focused contract tests:

- `ph1x_canonical_active_context_packet_default_and_typical_values_validate`
- `ph1x_canonical_human_conversation_directive_maps_existing_ph1x_directive`
- `ph1m_canonical_memory_evidence_packet_carries_memory_styles`
- `ph1m_canonical_memory_recall_request_wraps_existing_ph1m_recall_request`
- `ph1m_canonical_fresh_memory_handoff_represents_post_sleep_followup_evidence`
- `ph1m_canonical_memory_continuation_decision_variants_validate`

Validation completed:

- `cargo fmt`
- `cargo test -p selene_kernel_contracts ph1x_canonical -- --test-threads=1`
- `cargo test -p selene_kernel_contracts ph1m_canonical -- --test-threads=1`
- `cargo check`
- `cargo test -p selene_kernel_contracts -- --test-threads=1`
- `cargo test -p selene_engines ph1x -- --test-threads=1`
- `cargo test -p selene_engines ph1m -- --test-threads=1`
- `cargo test -p selene_adapter active_session_context -- --test-threads=1`
- `cargo test -p selene_adapter recent_archive_recall_does_not_pollute_active_context_after_answer -- --test-threads=1`
- `cargo test -p selene_adapter -- --test-threads=1`
- `cargo test -p selene_os -- --test-threads=1`
- `cargo test -p selene_engines -- --test-threads=1`
- `git diff --check`

## Stage 7 Readiness

READY_FOR_STAGE_7_CONTRACTS

Stage 7 can begin as an immutable evidence/storage build using these canonical packet refs.

Remaining non-blocking future work:

- Wire PH1.X runtime behavior to produce `ActiveContextPacket` and `HumanConversationDirective` as first-class evidence.
- Wire PH1.M runtime behavior to produce `MemoryEvidencePacket`, `FreshMemoryHandoff`, and `MemoryContinuationDecision`.
- Store the canonical refs immutably in Stage 7.
- Retire adapter-local context shortcuts after canonical PH1.X/PH1.M behavior replaces them and tests prove no regression.
