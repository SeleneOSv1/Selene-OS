# Selene OS Full System Build Context (Design + Build + Runtime)

Last updated: 2026-02-14
Pinned baseline commit: `1aeda83`
Audience: any new chat/session that must immediately help with Selene design and implementation.

## 1) What Selene Is
Selene OS is a contracts-first, orchestration-first Rust system.

Core laws that define the system:
- one orchestrator (`selene_os`) for cross-engine sequencing
- engines never call engines directly
- no simulation -> no side-effect execution
- identity is not authority
- memory is non-authoritative
- every critical decision is reason-coded and auditable

Authoritative law source:
- `docs/05_OS_CONSTITUTION.md`

## 2) Repository Topology
Top-level:
- `Cargo.toml` (workspace)
- `Cargo.lock`
- `README.md`
- `CONTRIBUTING.md`
- `crates/`
- `docs/`
- `scripts/`

Rust workspace members (`Cargo.toml`):
- `crates/selene_kernel_contracts`
- `crates/selene_os`
- `crates/selene_engines`
- `crates/selene_storage`
- `crates/selene_tools`
- `crates/selene_replay`

### 2.1 Crate Responsibilities
`crates/selene_kernel_contracts`
- shared runtime contract types + validators
- source files include: `common.rs`, `ph1*.rs` contracts for engines, work orders, simulation catalog, blueprints, etc.

`crates/selene_os`
- orchestrator runtime
- enforces gate order and simulation dispatch
- key modules: `ph1x.rs`, `simulation_executor.rs`, `ph1link.rs`, `ph1onb.rs`, `ph1position.rs`, `ph1l.rs`, `ph1capreq.rs`, `ph1explain.rs`, `ph1_voice_id.rs`, `ph1w.rs`

`crates/selene_engines`
- deterministic worker logic for perception/understanding/output assistance
- key modules: `ph1k.rs`, `ph1w.rs`, `ph1c.rs`, `ph1d.rs`, `ph1n.rs`, `ph1m.rs`, `ph1tts.rs`, `ph1_voice_id.rs`

`crates/selene_storage`
- persistence implementation + migrations + DB wiring tests
- key modules: `ph1f.rs`, `ph1j.rs`, `repo.rs`
- migrations: `crates/selene_storage/migrations/*.sql`
- integration tests: `crates/selene_storage/tests/**/db_wiring.rs`

`crates/selene_tools`
- read-only tool routing layer
- key module: `ph1e.rs`

`crates/selene_replay`
- replay binary scaffold (`src/main.rs`)

## 3) Documentation System (Canonical vs Non-Canonical)
Canonical design truth model source:
- `docs/00_DESIGN_TRUTH_OPTION_B.md`

### 3.1 Canonical Sources (must update in same change)
- engine inventory: `docs/07_ENGINE_REGISTRY.md`
- simulation inventory: `docs/08_SIMULATION_CATALOG.md`
- blueprint mapping index: `docs/09_BLUEPRINT_REGISTRY.md`
- blueprint process records: `docs/BLUEPRINTS/*.md`
- DB ownership summary: `docs/10_DB_OWNERSHIP_MATRIX.md`
- design lock status: `docs/11_DESIGN_LOCK_SEQUENCE.md`
- coverage state: `docs/COVERAGE_MATRIX.md`
- memory architecture narrative: `docs/12_MEMORY_ARCHITECTURE.md`
- detailed engine contracts: `docs/DB_WIRING/*.md`, `docs/ECM/*.md`
- kernel envelope contract: `docs/04_KERNEL_CONTRACTS.md`

### 3.2 Non-Canonical / Navigation / Evidence Docs
- `docs/00_INDEX.md` (navigation)
- `docs/01_ARCHITECTURE.md` (high-level architecture summary)
- `docs/02_BUILD_PLAN.md` (roadmap)
- `docs/03_BUILD_LEDGER.md` (append-only evidence/history)
- `docs/05_OS_CONSTITUTION.md` (law + pointers)
- `docs/06_ENGINE_MAP.md` (summary/runtime map)

## 4) Current Docs Inventory Snapshot
Counts at this snapshot:
- total docs files: `142`
- total docs lines: `16891`
- blueprint records: `15`
- DB wiring contracts: `55`
- ECM contracts: `54`

Primary directories:
- `docs/BLUEPRINTS`
- `docs/DB_WIRING`
- `docs/ECM`

Blueprint files currently present:
- `CAPREQ_MANAGE.md`
- `EMO_PROFILE_MANAGE.md`
- `LINK_DELIVER_INVITE.md`
- `LINK_INVITE.md`
- `LINK_OPEN_ACTIVATE.md`
- `MEMORY_FORGET_REQUEST.md`
- `MEMORY_QUERY.md`
- `MEMORY_REMEMBER_REQUEST.md`
- `MESSAGE_COMPOSE_AND_SEND.md`
- `ONB_BIZ_SETUP.md`
- `ONB_INVITED.md`
- `POSITION_MANAGE.md`
- `REMINDER_MANAGE.md`
- `TOOL_TIME_QUERY.md`
- `TOOL_WEATHER_QUERY.md`

## 5) Architecture and Runtime Wiring
Architecture summary source:
- `docs/01_ARCHITECTURE.md`
- `docs/06_ENGINE_MAP.md`

Global gate order:
1. Identity
2. STT
3. NLP understanding
4. Confirmation (if required)
5. Access
6. Simulation
7. Domain execution
8. Persist + audit

Voice path (high-level):
- `PH1.K -> PH1.W -> PH1.VOICE.ID -> PH1.C -> PH1.NLP -> PH1.X -> PH1.WRITE -> PH1.TTS`

Text path (high-level):
- `UI -> transcript_ok equivalent -> PH1.NLP -> PH1.X -> PH1.WRITE -> UI`

Cross-engine law:
- engines are workers only
- `selene_os` is the only cross-engine orchestrator

## 6) Engine Inventory and Status
Authoritative inventory:
- `docs/07_ENGINE_REGISTRY.md`

Current notable engine groups:
- Foundations: `PH1.F`, `PH1.J`, `SELENE_OS_CORE_TABLES`, `PBS_TABLES`, `SIMULATION_CATALOG_TABLES`, `ENGINE_CAPABILITY_MAPS_TABLES`, `ARTIFACTS_LEDGER_TABLES`
- Identity/Access: `PH1.L`, `PH1.VOICE.ID`, `PH1.ACCESS.001_PH2.ACCESS.002`, `PH1.POLICY`
- Perception/Understanding/Control core: `PH1.K`, `PH1.W`, `PH1.C`, `PH1.NLP`, `PH1.D`, `PH1.X`
- Output: `PH1.WRITE`, `PH1.TTS`
- Onboarding/Delivery/Tools: `PH1.E`, `PH1.BCAST`, `PH1.DELIVERY`, `PH1.ONBOARDING_SMS`, `PH1.LINK`, `PH1.ONB`, `PH1.POSITION`, `PH1.REM`
- Memory/Learning: `PH1.M`, `PH1.PERSONA`, `PH1.FEEDBACK`, `PH1.LEARN`, `PH1.PAE`, `PH1.KNOW`, `PH1.LEARN_FEEDBACK_KNOW`, `PH1.EMO.GUIDE`, `PH1.EMO.CORE`
- Capability requests: `PH1.CAPREQ`
- Extension assists and offline engines are also listed in registry and remain orchestrator-mediated.

## 7) Blueprint System
Authoritative mapping index:
- `docs/09_BLUEPRINT_REGISTRY.md`

Current registry state:
- all listed intents are `ACTIVE`
- one ACTIVE blueprint per intent_type

Active intents currently in registry:
- `LINK_INVITE`
- `LINK_DELIVER_INVITE`
- `LINK_OPEN_ACTIVATE`
- `ONB_INVITED`
- `ONB_BIZ_SETUP`
- `POSITION_MANAGE`
- `CAPREQ_MANAGE`
- `MESSAGE_COMPOSE_AND_SEND`
- `MEMORY_QUERY`
- `MEMORY_FORGET_REQUEST`
- `MEMORY_REMEMBER_REQUEST`
- `TOOL_TIME_QUERY`
- `TOOL_WEATHER_QUERY`
- `REMINDER_MANAGE`
- `EMO_PROFILE_MANAGE`

Blueprint discipline:
- every step `capability_id` must resolve to active ECM contracts
- side-effect steps must reference valid simulation IDs
- ACTIVE blueprints must be code-ready by design discipline

## 8) Simulation Catalog System
Authoritative inventory:
- `docs/08_SIMULATION_CATALOG.md`

Simulation catalog is large (3000+ lines) and includes:
- index table
- domain DB binding profiles
- detailed `### SIMULATION_ID` records

Important PH1.LINK simulation separation rules:
- `LINK_INVITE_SEND_COMMIT`, `LINK_INVITE_RESEND_COMMIT`, `LINK_DELIVERY_FAILURE_HANDLING_COMMIT` are legacy placeholders and must stay `LEGACY_DO_NOT_WIRE`
- active link delivery path ownership is `LINK_DELIVER_INVITE` via `PH1.BCAST + PH1.DELIVERY`
- `LINK_INVITE_EXPIRED_RECOVERY_COMMIT` is ACTIVE and state-only

## 9) PH1.LINK Canonical Contract (Critical Drift Zone)
Primary files:
- `docs/DB_WIRING/PH1_LINK.md`
- `docs/ECM/PH1_LINK.md`
- `docs/BLUEPRINTS/LINK_INVITE.md`
- `docs/BLUEPRINTS/LINK_OPEN_ACTIVATE.md`
- `docs/BLUEPRINTS/LINK_DELIVER_INVITE.md`
- `crates/selene_kernel_contracts/src/ph1link.rs`
- `crates/selene_storage/migrations/0012_ph1link_onboarding_draft_tables.sql`

Canonical invitee_type set:
- `COMPANY | CUSTOMER | EMPLOYEE | FAMILY_MEMBER | FRIEND | ASSOCIATE`

Canonical link-token lifecycle state set:
- `DRAFT_CREATED | SENT | OPENED | ACTIVATED | CONSUMED | REVOKED | EXPIRED | BLOCKED`

## 10) PH1.E / PH1.REM / PH1.EMO.GUIDE+PH1.EMO.CORE Closure State
`PH1.E`
- DB wiring + ECM docs complete
- tool blueprints complete (`TOOL_TIME_QUERY`, `TOOL_WEATHER_QUERY`)
- simulation ownership aligned in coverage matrix

`PH1.REM`
- 4-pack present (DB_WIRING + ECM + sim entries + blueprint)
- key blueprint: `docs/BLUEPRINTS/REMINDER_MANAGE.md`
- BCAST handoff path represented (`BCAST_MHP_FOLLOWUP`)

`PH1.EMO.GUIDE + PH1.EMO.CORE`
- concrete docs/sim entries + blueprint present
- key blueprint: `docs/BLUEPRINTS/EMO_PROFILE_MANAGE.md`
- tone-only guarantees documented in contracts

## 11) DB Wiring / Migration Backbone
DB wiring sequence source:
- `docs/DB_WIRING/00_DB_WIRING_DESIGN_LOCK_SEQUENCE.md`

Current sequence state:
- rows 1..26 are `PASS`
- current next engine: `none` (sequence scope complete)

Migrations currently present (`crates/selene_storage/migrations`):
- `0001_ph1f_foundation.sql`
- `0002_work_orders_core.sql`
- `0003_pbs_tables.sql`
- `0004_simulation_catalog_tables.sql`
- `0005_engine_capability_maps_tables.sql`
- `0006_artifacts_ledger_and_tool_cache.sql`
- `0007_ph1l_sessions_indexes.sql`
- `0008_ph1vid_voice_enrollment_tables.sql`
- `0009_access_instance_tables.sql`
- `0010_ph1k_audio_runtime_tables.sql`
- `0011_ph1w_wake_tables.sql`
- `0012_ph1link_onboarding_draft_tables.sql`
- `0013_capreq_tables.sql`

## 12) Build Plan and Lock Sequence
Build roadmap source:
- `docs/02_BUILD_PLAN.md`

Design lock sequence source:
- `docs/11_DESIGN_LOCK_SEQUENCE.md`

Current lock-sequence state:
- ordered items 1..10 are `LOCKED`
- readiness audit precondition is mandatory
- canonical readiness audit command:
  - `scripts/selene_design_readiness_audit.sh`

## 13) Coverage and Readiness State
Coverage source:
- `docs/COVERAGE_MATRIX.md`

Current state summary:
- no `TODO`, no `BLOCKER`, no `WIP` rows in matrix
- key rows `PH1.E`, `PH1.LINK`, `PH1.REM`, `PH1.EMO.GUIDE`, `PH1.EMO.CORE` are all fully `DONE`

Problem tracker source:
- `docs/13_PROBLEMS_TO_FIX.md`

Current tracker summary:
- items 1..19 marked fixed with proof logs

## 14) Runtime Verification Status
Executed runtime verification commands:
- `cargo test --workspace`
- `cargo test --workspace --release`

Current result snapshot:
- all unit/integration/doc tests passed
- failures: `0`
- repository remained clean post-run

## 15) Operational Commands for New Chat
From repo root:

```bash
# repo state
git branch --show-current
git rev-parse HEAD
git status --short

# design-readiness audit (canonical)
scripts/selene_design_readiness_audit.sh

# runtime verification
cargo test --workspace
cargo test --workspace --release
```

Focused discipline checks:

```bash
# coverage open items
rg -n "TODO|BLOCKER|WIP" docs/COVERAGE_MATRIX.md

# legacy token drift sweeps
# NOTE: Run scripts/selene_design_readiness_audit.sh for drift/banned-token sweeps; do not paste the regex here.

# registry critical rows
rg -n "^\| PH1\.E \||^\| PH1\.LINK \||^\| PH1\.REM \||^\| PH1\.EMO\.GUIDE \||^\| PH1\.EMO\.CORE \|" docs/COVERAGE_MATRIX.md docs/07_ENGINE_REGISTRY.md
```

## 16) Mandatory Update Discipline for Any Future Change
If you change any engine/workflow contract, update in the same change set:
1. `docs/07_ENGINE_REGISTRY.md`
2. `docs/10_DB_OWNERSHIP_MATRIX.md`
3. `docs/COVERAGE_MATRIX.md`
4. `docs/08_SIMULATION_CATALOG.md` (if simulation changes)
5. `docs/09_BLUEPRINT_REGISTRY.md` + `docs/BLUEPRINTS/*.md` (if blueprint changes)
6. `docs/11_DESIGN_LOCK_SEQUENCE.md` (if lock status changes)
7. relevant `docs/DB_WIRING/*.md` and/or `docs/ECM/*.md`

Hard rules:
- no deferred canonical updates
- no duplicate inventory truth in non-canonical docs

## 17) Non-Negotiable Engineering Constraints
From constitution and repo conventions:
- Rust-only production implementation flow
- no engine-to-engine calls
- no side-effect path without simulation gate
- fail-closed posture when requirements are unmet
- reason-coded auditability for critical decisions
- snake_case fields and stable identifiers

## 18) Fast Start Reading Order for a New Chat
1. `docs/00_INDEX.md`
2. `docs/00_DESIGN_TRUTH_OPTION_B.md`
3. `docs/07_ENGINE_REGISTRY.md`
4. `docs/08_SIMULATION_CATALOG.md`
5. `docs/09_BLUEPRINT_REGISTRY.md`
6. `docs/COVERAGE_MATRIX.md`
7. `docs/11_DESIGN_LOCK_SEQUENCE.md`
8. `docs/13_PROBLEMS_TO_FIX.md`
9. `docs/14_NEW_CHAT_SYSTEM_CONTEXT.md`
10. this file (`docs/15_FULL_SYSTEM_BUILD_CONTEXT.md`)

Then subsystem deep dives as needed:
- kernel envelopes: `docs/04_KERNEL_CONTRACTS.md`
- runtime flow summary: `docs/06_ENGINE_MAP.md`
- memory architecture: `docs/12_MEMORY_ARCHITECTURE.md`
- PH1.LINK: `docs/DB_WIRING/PH1_LINK.md`, `docs/ECM/PH1_LINK.md`, `crates/selene_kernel_contracts/src/ph1link.rs`
- PH1.REM: `docs/DB_WIRING/PH1_REM.md`, `docs/ECM/PH1_REM.md`, `docs/BLUEPRINTS/REMINDER_MANAGE.md`
- PH1.EMO.GUIDE: `docs/DB_WIRING/PH1_EMO_GUIDE.md`, `docs/ECM/PH1_EMO_GUIDE.md`
- PH1.EMO.CORE: `docs/DB_WIRING/PH1_EMO_CORE.md`, `docs/ECM/PH1_EMO_CORE.md`, `docs/BLUEPRINTS/EMO_PROFILE_MANAGE.md`

## 19) Practical Guidance for Build/Design Help in New Chats
When a new chat is asked to implement a change, it should:
1. Identify canonical owner docs for the changed concept.
2. Verify cross-layer parity (kernel contracts <-> DB wiring <-> ECM <-> sim catalog <-> blueprint).
3. Apply minimum viable change.
4. Run proofs (rg/parity sweeps) and runtime tests (`cargo test` scope-appropriate).
5. Update `docs/13_PROBLEMS_TO_FIX.md` if the task is part of tracked closure work.

This process prevents design drift and keeps build artifacts aligned with execution contracts.
