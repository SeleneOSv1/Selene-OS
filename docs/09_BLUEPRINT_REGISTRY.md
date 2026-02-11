# Blueprint Registry (Authoritative)

This file is the authoritative registry index of Selene Process Blueprints (PBS).*.

Hard rules
- No blueprint match -> no process starts.
- Exactly one ACTIVE process per `intent_type`.
- Any step with side effects must reference a `simulation_id` in `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/08_SIMULATION_CATALOG.md`. No Simulation -> No Execution.
- Every executable step must bind explicit `engine_id + capability_id` from ACTIVE ECM docs.
- Detailed blueprint records live only in `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BLUEPRINTS/*.md` (no inline duplicate records in this registry).

## Blueprint Registry Schema Lock (Item 6)

Status: `LOCKED`

Locked schema rules:
- Every blueprint record must provide explicit `purpose`, `ordered_steps`, and `simulation_requirements`.
- `confirmation_points` must be explicit.
- `required_inputs` and `success_output_schema` must be explicit in each process record.
- No `TBD` placeholders are allowed in authoritative blueprint records.

## Registry Index (Authoritative Mapping)

| intent_type | process_id | version | status | record_path |
|---|---|---|---|---|
| LINK_INVITE | LINK_INVITE | v1 | ACTIVE | `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BLUEPRINTS/LINK_INVITE.md` |
| LINK_OPEN_ACTIVATE | LINK_OPEN_ACTIVATE | v1 | ACTIVE | `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BLUEPRINTS/LINK_OPEN_ACTIVATE.md` |
| ONB_INVITED | ONB_INVITED | v1 | DRAFT | `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BLUEPRINTS/ONB_INVITED.md` |
| ONB_BIZ_SETUP | ONB_BIZ_SETUP | v1 | DRAFT | `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BLUEPRINTS/ONB_BIZ_SETUP.md` |
| POSITION_MANAGE | POSITION_MANAGE | v1 | DRAFT | `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BLUEPRINTS/POSITION_MANAGE.md` |
| CAPREQ_MANAGE | CAPREQ_MANAGE | v1 | ACTIVE | `/Users/xiamo/Documents/A-Selene/Selene-OS/docs/BLUEPRINTS/CAPREQ_MANAGE.md` |
