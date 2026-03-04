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
4. EvidencePacket
5. SynthesisPacket
6. WritePacket
7. ComparisonPacket
8. RiskPacket
9. EnterpriseReportPacket (stub)
10. AuditPacket
11. ComputationPacket
12. VisionToolRequestPacket
13. VisionEvidencePacket
14. MergePacket

## Common Required Fields (all packets)
- `schema_version`
- `produced_by`
- `intended_consumers`
- `created_at_ms`
- `trace_id`

## Consumer Expected Versions
All Run 1 packet consumers are locked to `1.0.0` in `PACKET_SCHEMAS.json`.

## Schema Change Policy
See `BACKWARD_COMPAT_MATRIX.md`.
