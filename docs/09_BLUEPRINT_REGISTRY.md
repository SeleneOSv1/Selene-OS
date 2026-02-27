# Blueprint Registry (Authoritative)

This file is the authoritative registry index of Selene Process Blueprints (PBS).*.

Hard rules
- No blueprint match -> no process starts.
- Exactly one ACTIVE process per `intent_type`.
- Any step with side effects must reference a `simulation_id` in `docs/08_SIMULATION_CATALOG.md`. No Simulation -> No Execution.
- Every executable step must bind explicit `engine_id + capability_id` from ACTIVE ECM docs.
- Detailed blueprint records live only in `docs/BLUEPRINTS/*.md` (no inline duplicate records in this registry).

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
| LINK_INVITE | LINK_INVITE | v1 | ACTIVE | `docs/BLUEPRINTS/LINK_INVITE.md` |
| LINK_DELIVER_INVITE | LINK_DELIVER_INVITE | v1 | ACTIVE | `docs/BLUEPRINTS/LINK_DELIVER_INVITE.md` |
| LINK_OPEN_ACTIVATE | LINK_OPEN_ACTIVATE | v1 | ACTIVE | `docs/BLUEPRINTS/LINK_OPEN_ACTIVATE.md` |
| ONB_INVITED | ONB_INVITED | v1 | ACTIVE | `docs/BLUEPRINTS/ONB_INVITED.md` |
| ONB_BIZ_SETUP | ONB_BIZ_SETUP | v1 | ACTIVE | `docs/BLUEPRINTS/ONB_BIZ_SETUP.md` |
| ONB_SCHEMA_MANAGE | ONB_SCHEMA_MANAGE | v1 | ACTIVE | `docs/BLUEPRINTS/ONB_SCHEMA_MANAGE.md` |
| ONB_REQUIREMENT_BACKFILL | ONB_REQUIREMENT_BACKFILL | v1 | ACTIVE | `docs/BLUEPRINTS/ONB_REQUIREMENT_BACKFILL.md` |
| POSITION_MANAGE | POSITION_MANAGE | v1 | ACTIVE | `docs/BLUEPRINTS/POSITION_MANAGE.md` |
| CAPREQ_MANAGE | CAPREQ_MANAGE | v1 | ACTIVE | `docs/BLUEPRINTS/CAPREQ_MANAGE.md` |
| ACCESS_SCHEMA_MANAGE | ACCESS_SCHEMA_MANAGE | v1 | ACTIVE | `docs/BLUEPRINTS/ACCESS_SCHEMA_MANAGE.md` |
| ACCESS_ESCALATION_VOTE | ACCESS_ESCALATION_VOTE | v1 | ACTIVE | `docs/BLUEPRINTS/ACCESS_ESCALATION_VOTE.md` |
| ACCESS_INSTANCE_COMPILE_REFRESH | ACCESS_INSTANCE_COMPILE_REFRESH | v1 | ACTIVE | `docs/BLUEPRINTS/ACCESS_INSTANCE_COMPILE_REFRESH.md` |
| MESSAGE_COMPOSE_AND_SEND | MESSAGE_COMPOSE_AND_SEND | v1 | ACTIVE | `docs/BLUEPRINTS/MESSAGE_COMPOSE_AND_SEND.md` |
| MEMORY_QUERY | MEMORY_QUERY | v1 | ACTIVE | `docs/BLUEPRINTS/MEMORY_QUERY.md` |
| MEMORY_FORGET_REQUEST | MEMORY_FORGET_REQUEST | v1 | ACTIVE | `docs/BLUEPRINTS/MEMORY_FORGET_REQUEST.md` |
| MEMORY_REMEMBER_REQUEST | MEMORY_REMEMBER_REQUEST | v1 | ACTIVE | `docs/BLUEPRINTS/MEMORY_REMEMBER_REQUEST.md` |
| TOOL_TIME_QUERY | TOOL_TIME_QUERY | v1 | ACTIVE | `docs/BLUEPRINTS/TOOL_TIME_QUERY.md` |
| TOOL_WEATHER_QUERY | TOOL_WEATHER_QUERY | v1 | ACTIVE | `docs/BLUEPRINTS/TOOL_WEATHER_QUERY.md` |
| TOOL_WEB_SEARCH | TOOL_WEB_SEARCH | v1 | ACTIVE | `docs/BLUEPRINTS/TOOL_WEB_SEARCH.md` |
| TOOL_NEWS | TOOL_NEWS | v1 | ACTIVE | `docs/BLUEPRINTS/TOOL_NEWS.md` |
| TOOL_URL_FETCH_AND_CITE | TOOL_URL_FETCH_AND_CITE | v1 | ACTIVE | `docs/BLUEPRINTS/TOOL_URL_FETCH_AND_CITE.md` |
| TOOL_DOCUMENT_UNDERSTAND | TOOL_DOCUMENT_UNDERSTAND | v1 | ACTIVE | `docs/BLUEPRINTS/TOOL_DOCUMENT_UNDERSTAND.md` |
| TOOL_PHOTO_UNDERSTAND | TOOL_PHOTO_UNDERSTAND | v1 | ACTIVE | `docs/BLUEPRINTS/TOOL_PHOTO_UNDERSTAND.md` |
| TOOL_DATA_ANALYSIS | TOOL_DATA_ANALYSIS | v1 | ACTIVE | `docs/BLUEPRINTS/TOOL_DATA_ANALYSIS.md` |
| TOOL_DEEP_RESEARCH | TOOL_DEEP_RESEARCH | v1 | ACTIVE | `docs/BLUEPRINTS/TOOL_DEEP_RESEARCH.md` |
| REMINDER_MANAGE | REMINDER_MANAGE | v1 | ACTIVE | `docs/BLUEPRINTS/REMINDER_MANAGE.md` |
| EMO_PROFILE_MANAGE | EMO_PROFILE_MANAGE | v1 | ACTIVE | `docs/BLUEPRINTS/EMO_PROFILE_MANAGE.md` |
