# Selene OS

Selene OS is a contracts-first project with a Rust workspace scaffold and documentation-first build plan.

## PH1.F Database Spine

`PH1.F` is Selene's PostgreSQL persistence engine. It stores durable truth for:
- user/company records
- onboarding drafts and link token mappings (`token_id -> draft_id`)
- required/optional attribute state by tenant schema version
- conversation/audit/memory ledgers

In Selene, `attributes` and `data fields` mean the same thing (for example: name, role, department, salary tier ref, start date, phone).

How field collection works:
1. Creator fills known fields first into the onboarding draft.
2. Selene computes `missing_required_fields` deterministically from tenant schema (no LLM guessing).
3. Invitee is asked only missing required fields (never ask twice).
4. If required fields remain incomplete, Selene schedules reminders through `PH1.REM.001`.
5. When required fields are complete, COMMIT finalizes the record atomically and consumes the token idempotently.

Start here: [docs/00_INDEX.md](docs/00_INDEX.md)
Current ordered "what's next" checklist: [docs/11_DESIGN_LOCK_SEQUENCE.md](docs/11_DESIGN_LOCK_SEQUENCE.md)

## Local PostgreSQL Wiring (Hard-Isolated)

Bootstrap a dedicated Postgres instance for Selene OS only (separate cluster, port, role, DB, migrations):

```bash
./scripts/dev_postgres_setup.sh
```

Re-run migrations only:

```bash
./scripts/dev_postgres_migrate.sh
```

Local DB connection env is written to:

```text
.dev/db.env
```

The setup script also writes `.env.local` and isolates the cluster under `.dev/postgres` (default port `55432`).
