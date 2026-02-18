# PH1_ENDPOINT DB Wiring (Design vNext)

## A) Engine Header
- engine_id: PH1.ENDPOINT
- layer: Perception
- authority: Non-Authoritative
- role: Streaming endpoint boundary assist for capture/transcript alignment and turn segmentation hints
- placement: TURN_OPTIONAL

## B) Ownership
- Tables owned: NONE (vNext design)
- Reads:
  - Structured, OS-supplied VAD windows from PH1.K runtime signals.
  - Optional, OS-supplied PH1.LISTEN adaptation hints (`environment_profile_ref`, selected capture/endpoint profile) after LISTEN filter validation.
  - Structured transcript timing/token metadata from PH1.C routing path.
  - Optional previous segment-selection token from Selene OS turn state.
- Writes: NONE (no direct persistence in vNext)

## C) Hard Boundaries
- Non-authoritative and non-executing; advisory output only.
- Must never grant authority, alter permissions, or bypass Access + Simulation ordering.
- Must never perform side effects, tool execution, or engine-to-engine direct calls.
- Hard perception rule: endpoint hints may refine boundaries only and must never mutate transcript meaning.
- Hard fail-closed rule: boundary-score drift must stop endpoint handoff before PH1.C finalization.

## D) Wiring
- Invoked_by: Selene OS step after PH1.K VAD windows and before PH1.C transcript finalization.
- Inputs_from:
  - PH1.K: bounded VAD windows (`t_start_ms`, `t_end_ms`, `vad_confidence`, `speech_likeness`, trailing silence).
  - PH1.LISTEN (optional): validated capture/endpoint hint metadata (`LISTEN_SIGNAL_FILTER=OK` only).
  - PH1.C context: transcript token estimate and turn timing metadata.
- Outputs_to:
  - endpoint segment hints bundle to Selene OS.
  - Selene OS forwards selected boundary hint to PH1.C and optional boundary metadata back to PH1.K.
- Invocation_condition: OPTIONAL(capture boundary refinement)
- Deterministic sequence:
  - `ENDPOINT_HINTS_BUILD`: build ordered segment hints from bounded VAD windows.
  - `ENDPOINT_BOUNDARY_SCORE`: validate selected-vs-ordered hint consistency and no-repeat drift guard.
  - If `validation_status != OK`, Selene OS fails closed and does not forward endpoint hints downstream.
- Not allowed:
  - Engine-to-engine direct calls.
  - Any execution commit or authority mutation.
  - Any bypass of Selene OS orchestration.

## E) Related Engine Boundaries
- PH1.K remains source of runtime perception windows; PH1.ENDPOINT does not own stream capture/runtime state.
- PH1.LISTEN may provide optional adaptation hints, but PH1.ENDPOINT still owns boundary scoring and fail-closed boundary validation.
- PH1.C remains owner of transcript gate pass/reject decisions; PH1.ENDPOINT only provides pre-finalization boundary hints.

## F) Acceptance Tests
- AT-ENDPOINT-01: Selene OS can invoke `ENDPOINT_HINTS_BUILD` and output is schema-valid.
- AT-ENDPOINT-02: Segment hint ordering is deterministic for identical VAD inputs.
- AT-ENDPOINT-03: Hint budget limits are enforced and fail closed when exceeded.
- AT-ENDPOINT-04: Boundary-score validation fails closed on selected-segment drift.
