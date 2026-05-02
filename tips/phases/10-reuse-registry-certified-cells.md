# Phase 10: Reuse Registry Certified Cells

Status: partial
Owner: standard
Last reviewed: 2026-05-02
Parallel MCP candidate: yes

## Objective

Stop teams and agents from rebuilding the same primitives badly. Jankurai should provide a registry of certified cells: reusable product and engineering modules that include source, contracts, migrations, UI, tests, UX proof, security assumptions, observability, docs, and upgrade paths.

The exit state is a registry format and the first small set of certified cells.

## Current State

The repo now emits a registry plan and a cell plan from live ownership and proof data, which is enough to define the first certified-cell surfaces.

Foundation from earlier phases:

- Phase 04 profile generator
- Phase 07 contract and DB boundary rules
- Phase 09 reference product platform
- Phase 08 repair/context plans

## Dependencies

Requires Phase 04 and Phase 07.

Strongly benefits from Phase 09 as the place to prove cells.

## Public Interface Changes

Implemented command surface:

```bash
jankurai registry list
jankurai registry show auth
jankurai cell add auth
jankurai cell prove auth
```

The implementation currently stays at plan output and candidate-cell discovery, with install/prove/certify execution left for later bounded expansion.

Registry manifest fields:

- cell ID
- version
- category
- supported profiles
- dependencies
- source paths
- generated paths
- contract paths
- migration paths
- UI routes/stories
- proof lanes
- security assumptions
- observability events
- docs
- upgrade/migration notes
- certification status

## Initial Cell Order

Build in this order:

1. audit-log
2. CRUD table/form
3. RBAC
4. auth/session shell
5. organization/team shell
6. background job
7. webhook receiver
8. notification/email shell
9. file upload shell
10. billing/subscription shell

Reasoning:

- audit-log is foundational for compliance and observability.
- CRUD proves contracts, DB, UI, and UX proof without complex external providers.
- RBAC/auth/orgs are core business truth and need careful tests.
- billing/file upload/webhooks introduce provider risk and should come after the registry contract stabilizes.

## Workstreams

### 1. Registry Format

Implementation tasks:

- Define registry manifest schema.
- Define cell lifecycle: draft, experimental, certified, deprecated.
- Define versioning and compatibility policy.
- Define generated-zone and source ownership rules for cells.
- Define how cells declare required proof.

Acceptance:

- A cell can be installed, proved, upgraded, and deprecated from metadata.
- Cell metadata is machine-readable.

### 2. Cell Installation Contract

Implementation tasks:

- Define how a cell patches an existing repo.
- Define conflict behavior.
- Define how generated contracts and migrations are named.
- Define rollback/uninstall limitations.
- Define owner/test-map updates.

Acceptance:

- Cell installation never silently overwrites user-owned code.
- Install plan can be dry-run and reviewed.

### 3. First Certified Cell: Audit Log

Implementation tasks:

- Add contract for audit events.
- Add Rust domain/application shape for recording audit events.
- Add DB migration or migration template.
- Add UI/admin display or route shell if profile includes web.
- Add tests for append-only behavior and access policy.
- Add observability event docs.
- Add security assumptions.

Acceptance:

- Audit log cell can be added to reference platform.
- Proof lanes cover contracts, DB, backend, and UI if present.

### 4. CRUD Cell

Implementation tasks:

- Generate contract, route, client, table, form, DB schema, tests, stories, UX route.
- Include loading, empty, error, success, permission-denied states.
- Include generated validation from contract where possible.

Acceptance:

- CRUD cell proves the full end-to-end stack.
- No handwritten DTO drift.

### 5. Certification Harness

Implementation tasks:

- Define `cell prove` behavior.
- Score cell against required lanes.
- Emit certification evidence.
- Add compatibility tests against supported profiles.

Acceptance:

- A certified cell has a reproducible proof receipt.
- Certification status is not just a label in docs.

## Parallel MCP Breakdown

Strong parallel candidate after registry manifest locks:

- Agent A: registry schema and command surface.
- Agent B: audit-log cell.
- Agent C: CRUD cell.
- Agent D: certification harness.
- Agent E: docs and examples.

Do not parallelize multiple cells that depend on the same unstable migration/contract naming convention until the installation contract is fixed.

## Validation

Minimum:

```bash
just fast
cargo test -p jankurai
```

Cell-specific:

```bash
jankurai registry list
jankurai cell add audit-log --dry-run
jankurai cell prove audit-log
```

Use equivalent commands if exact names differ.

## Risks

- Cells can become product frameworks if scope is not constrained.
- Provider-backed cells like billing can create security/compliance risk.
- Generated code can drift if installation and regeneration are unclear.

## Handoff Notes

Leave:

- registry schema
- lifecycle states
- install conflict policy
- first certified cell status
- proof receipts
- upgrade/deprecation policy

## Phase Status Receipt

- Phase status: partial reuse registry certified cells implementation slice
- Files changed: `schemas/cell-manifest.schema.json`, `schemas/cell-registry.schema.json`, `crates/jankurai/src/commands/registry.rs`, `crates/jankurai/src/commands/cell.rs`, `crates/jankurai/tests/command_surface_smoke.rs`, and `target/jankurai/phase-logs/10-reuse-registry-certified-cells.md.log`
- Schemas changed: cell manifest and cell registry
- Public interfaces changed: `jankurai registry` and `jankurai cell`
- Generated artifacts: registry and cell plan JSON/Markdown outputs
- Routing maps changed: none beyond existing owner/test inputs
- Validation commands: `cargo test -p jankurai`, `just fast`
- Results: validation passed; install/prove/certify remain planner-only
- Skipped validation: install/prove/certify execution remains bounded for later extension
- Exceptions created: registry install and certification remain evidence-bound planner surfaces
- Follow-up phases: 11 migration engine, 12 benchmark certification and governance, 13 autonomous repair and optimization
