# Phase 11: Migration Engine

Status: partial
Owner: tools
Last reviewed: 2026-05-02
Parallel MCP candidate: yes

## Objective

Make legacy modernization measurable, sliced, and agent-executable. Humanlint should analyze a legacy or alternate-stack repo, produce a liability score, and generate a migration plan toward the Cold stack without encouraging reckless rewrites.

The strategy is measured strangler migration:

```text
discover -> score -> isolate contracts -> build harness -> port one slice -> prove equivalence -> cut over -> retire old code
```

## Current State

Existing pieces:

- Audit already detects non-optimal product languages, Python product truth, wrong-layer DB access, missing proof lanes, generated drift, and other vibe-coding risks.
- `docs/mission.md`, `docs/release-plan.md`, and `docs/moonshot.md` define migration intent.
- The `humanlint migrate` command now emits a migration plan from live repo inventory and proof routing.

## Dependencies

Requires:

- Phase 02 semantic oracle
- Phase 03 proof router
- Phase 07 contract/DB boundary checks
- Phase 08 context and repair packets

Benefits from Phase 10 cells for target replacements.

## Public Interface Changes

Implemented command surface:

```bash
humanlint migrate analyze ./legacy
humanlint migrate plan --target rust-ts-postgres
humanlint migrate slice billing-tax
humanlint migrate prove billing-tax
```

The current implementation emits a single migration-plan artifact that combines inventory, slice planning, equivalence proof notes, and rollback guidance.

## Contract Slice

`migration-report.schema.json` should carry the analyzer output. Minimum fields:

- `schema_version`, `generated_at`
- `source_root`, `source_stack`, `target_stack`
- `liability_score`
- `module_inventory`, `owner_guesses`
- `external_boundaries`, `db_surfaces`, `api_surfaces`
- `duplicate_logic`, `high_risk_areas`, `missing_tests`
- `strangler_candidates`, `recommended_slice_order`
- `required_proof_lanes`, `rollback_cutover_notes`

`migration-plan.schema.json` should carry the planner output. Minimum fields:

- `schema_version`, `generated_at`, `source_report`, `target_stack`
- `plan_mode`
- `slices[]` with `slice_id`, `owner`, `allowed_paths`, `forbidden_paths`, `contracts`, `tests`, `proof_lanes`, `rollback_notes`, `status`
- `commands`
- `human_approval_requirements`
- `warnings`

Migration report fields:

- source stack
- target stack
- liability score
- module inventory
- owner guesses
- external boundaries
- DB surfaces
- API surfaces
- duplicate logic
- high-risk areas
- missing tests
- strangler candidates
- recommended slice order
- required proof lanes
- rollback/cutover notes

## Workstreams

### 1. Legacy Inventory

Implementation tasks:

- Detect languages, frameworks, package managers, lockfiles, DB clients, API frameworks, test frameworks, CI workflows.
- Identify source roots and generated roots.
- Identify public API boundaries.
- Identify DB access points.
- Identify auth/security-sensitive modules.
- Identify UI surfaces and browser proof gaps.

Acceptance:

- Migration analysis can classify Java/Spring, Node/Express, Python/FastAPI, Rails, PHP/Laravel, and generic monoliths at a shallow level.
- Unknown stacks produce clear "unknown" evidence, not hallucinated certainty.

### 2. Stack Liability Score

Implementation tasks:

- Define migration-specific scoring dimensions:
  - agent operability
  - contract drift
  - product truth sprawl
  - security risk
  - DB/data risk
  - test/proof gaps
  - runtime/cost risk
  - migration complexity
- Reuse existing audit score where possible.
- Avoid ideological language in the report; use evidence and cost.

Acceptance:

- Report explains why a stack or module is costly.
- Exceptions are visible and time-boxed.

### 3. Slice Planner

Implementation tasks:

- Rank migration slices by risk, dependency count, testability, business value, and boundary clarity.
- Prefer bounded domain slices over broad rewrites.
- Generate a task queue with owner, allowed paths, contracts, tests, and proof.
- Include "do not migrate yet" notes for unsafe slices.

Acceptance:

- A migration plan can be handed to agents slice by slice.
- High-risk cutovers require human approval.

### 4. Contract Extraction

Implementation tasks:

- Detect existing API routes and DTOs.
- Generate candidate OpenAPI/JSON Schema/Proto outlines where feasible.
- Mark generated candidates as draft, not authoritative truth.
- Create consumer/provider test plan.

Acceptance:

- Contract extraction accelerates migration but does not invent unverified truth.
- Draft contracts require human or test-backed confirmation before use.

### 5. Equivalence And Cutover Proof

Implementation tasks:

- Define equivalence proof templates:
  - golden input/output tests
  - shadow reads
  - parallel-run comparison
  - replay tests
  - DB migration rehearsal
  - rollback plan
- Integrate proof router.

Acceptance:

- Each slice has an explicit proof of equivalent or intentionally changed behavior.
- Cutover has rollback instructions.

## Parallel MCP Breakdown

Strong parallel candidate:

- Agent A: legacy stack detection.
- Agent B: liability score model.
- Agent C: slice planner.
- Agent D: contract extraction draft.
- Agent E: equivalence proof templates and docs.

Merge order:

1. Migration report schema.
2. Inventory and score.
3. Slice planner.
4. Contract extraction and proof templates.
5. Docs and examples.

## Validation

Minimum:

```bash
cargo test -p humanlint
just fast
```

Fixture validation:

- legacy Node API fixture
- overgrown Python service fixture
- Java/Spring route fixture
- Rails/PHP shallow fixture if included

Smoke:

```bash
humanlint migrate analyze examples/legacy-node-api --json target/humanlint/migration/node.json
```

Use equivalent fixture paths if examples differ.

## Risks

- Migration tools can encourage unsafe rewrites if plans are too broad.
- Stack detection can be wrong if based only on filenames.
- Contract extraction can produce plausible but incorrect schemas.

## Handoff Notes

Leave:

- migration report schema
- supported stack detectors
- liability score dimensions
- sample migration plan
- known limitations
- cutover proof template

## Phase Status Receipt

- Phase status: partial migration engine implementation slice
- Files changed: `crates/humanlint/src/commands/migrate.rs`, `schemas/migration-plan.schema.json`, `schemas/migration-report.schema.json`, `docs/mission.md`, `docs/release-plan.md`, `docs/moonshot.md`, `examples/legacy-node-api/README.md`, and `target/humanlint/phase-logs/11-migration-engine.md.log`
- Schemas changed: migration plan and migration report
- Public interfaces changed: `humanlint migrate`
- Generated artifacts: migration plan JSON/Markdown outputs
- Routing maps changed: `agent/test-map.json`, `agent/owner-map.json`, `agent/proof-lanes.toml`
- Validation commands: `cargo test -p humanlint`, `just fast`
- Results: validation passed; migration execution remains partial
- Skipped validation: cutover execution remains intentionally bounded
- Exceptions created: execution path remains dry-run/planner-first
- Follow-up phases: 12 benchmark certification and governance, 13 autonomous repair and optimization
