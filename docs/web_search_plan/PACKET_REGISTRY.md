# Packet Registry (Run 1 Foundation Lock)

This registry is the canonical authority for web research lane packet contracts.

## Hard Rules
- Read-only lane only. No SimulationExecutor involvement.
- Fail-closed by default.
- Unknown packet name, unknown schema version, missing required fields, or unknown top-level fields fail validation.
- `additional_properties` defaults to `false`.

## Canonical Packet Set
1. TurnInputPacket
2. SearchAssistPacket
3. ToolRequestPacket
4. VisionToolRequestPacket
5. EvidencePacket
6. VisionEvidencePacket
7. SynthesisPacket
8. WritePacket
9. ComparisonPacket (stub)
10. RiskPacket (stub)
11. EnterpriseReportPacket (stub)
12. AuditPacket

## Common Required Fields (all packets)
- `schema_version`
- `produced_by`
- `intended_consumers`
- `created_at_ms`
- `trace_id`

## Consumer Expected Versions
All Run 1 packet consumers are locked to `1.0.0` in `PACKET_SCHEMAS.json`.

## Vision Packet Contract Extensions (Run 12A)
- `VisionToolRequestPacket` required fields:
  - common fields (`schema_version`, `produced_by`, `intended_consumers`, `created_at_ms`, `trace_id`)
  - `mode` (`image_ocr|image_objects|image_analyze|video_transcribe|video_keyframes|video_analyze`)
  - `asset_ref` (`asset_hash`, `locator`, `mime_type`, `size_bytes`)
  - `options` (`safe_mode`, optional `language_hint|max_frames|frame_stride_ms`)
  - `budgets` (`timeout_ms`, `max_bytes`)
  - `policy_snapshot_id`
- `VisionEvidencePacket` required fields:
  - common fields
  - `asset_ref`, `retrieved_at_ms`, `provider_runs`, `outputs`, `confidence_summary`
  - `reason_codes`, `packet_hashes`, `output_hash`

## Schema Change Policy
See `BACKWARD_COMPAT_MATRIX.md`.
