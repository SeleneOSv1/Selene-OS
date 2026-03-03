# Turn State Machine (Run 1 Foundation Lock)

## Allowed States
- TURN_ACCEPTED
- INPUT_PARSED
- INTENT_CLASSIFIED
- PLAN_SELECTED
- RETRIEVAL_EXECUTED
- EVIDENCE_LOCKED
- SYNTHESIS_READY
- OUTPUT_RENDERED
- AUDIT_COMMITTED
- TURN_COMPLETED
- TURN_FAILED_CLOSED

## Allowed Transition Backbone
- TURN_ACCEPTED -> INPUT_PARSED -> INTENT_CLASSIFIED -> PLAN_SELECTED
- PLAN_SELECTED -> RETRIEVAL_EXECUTED -> EVIDENCE_LOCKED -> SYNTHESIS_READY -> OUTPUT_RENDERED -> AUDIT_COMMITTED -> TURN_COMPLETED
- Any non-terminal -> TURN_FAILED_CLOSED

## Gate Order (Locked)
1. Session/identity gate
2. Input quality gate
3. Intent/mode gate
4. Policy/access/quota/cost gate
5. Retrieval execution gate
6. Evidence integrity gate
7. Synthesis gate
8. Output gate
9. Audit/persistence gate

## Validator Rules
- Transition sequences must be legal.
- TURN_FAILED_CLOSED requires at least one reason code.
- AUDIT_COMMITTED can occur only after OUTPUT_RENDERED or TURN_FAILED_CLOSED.
