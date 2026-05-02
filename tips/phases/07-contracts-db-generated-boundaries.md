# Phase 07: Contracts DB And Generated Boundaries

Status: partial
Owner: standard
Last reviewed: 2026-05-02
Parallel MCP candidate: yes

## Objective

Make boundary truth executable. Jankurai must enforce that public APIs, event schemas, generated clients, database migrations, and durable invariants are declared, generated, tested, and owned.

The exit state is a contracts and DB lane that makes handwritten drift, wrong-layer persistence, and unsafe migrations visible.

## Current State

Existing policy:

- `docs/agent-native-standard.md` defines contracts, generated zones, DB truth, and layer boundaries.
- `agent/generated-zones.toml` exists.
- `agent/boundaries.toml` exists.
- Audit detects generated-zone risks, contract surfaces, direct DB wrong-layer hits, destructive SQL hits, and streaming runtime drift.
- **`jankurai doctor`** validates committed **`agent/boundaries.toml`** against **`schemas/boundaries.schema.json`** (TOML parsed and checked as JSON-shaped instance). **`ArtifactSchema::Boundaries`** in Rust; tests in `crates/jankurai/tests/boundaries_manifest_smoke.rs`.
- **`jankurai audit`** ingests the same manifest when present and schema-valid: **`boundaries.artifact`** holds a compact summary (path, content fingerprint, stack id/version, queue path counts, streaming-exception count). Invalid or missing files leave **`artifact`** omitted. Implementation in `crates/jankurai/src/audit/boundaries_artifact.rs`; tests in `crates/jankurai/tests/boundaries_audit_ingest_smoke.rs`; **`schemas/repo-score.schema.json`** documents **`boundaries`**. Score caps unchanged. **Rendered output:** `crates/jankurai/src/render.rs` and `crates/jankurai/src/report/github.rs` surface the same digest in **`repo-score.md`** and CI step summaries (see workstream 5 / slice 3).
- **`jankurai prove`** may record optional **`boundaries_manifest_path`** (`agent/boundaries.toml`) on **`evidence-index.json`** when that file exists (`schemas/evidence-index.schema.json`, `crates/jankurai/src/commands/proof.rs`).
- **`jankurai audit`** (Phase 07 slice 4) requires every **`[[zone]]`** in **`agent/generated-zones.toml`** to declare non-empty **`path`**, **`source`**, and **`command`** (trimmed). Incomplete rows yield one aggregated **HLT-002-GENERATED-MUTATION** finding on the manifest path and count toward the **`generated-zone-mutation-risk`** cap. Implementation in **`crates/jankurai/src/audit/scan.rs`** (`generated_zone_manifest_metadata_issues`); tests in **`crates/jankurai/tests/generated_zones_manifest_smoke.rs`**.

Gaps:

- Contract parsing and diffing are limited beyond boundary manifest shape (audit digest does not replace deep contract validation).
- Generated-zone reproducibility is not fully enforced.
- DB migration safety is mostly pattern-based.
- Event contract boundaries are policy-level, not deeply validated.

## Dependencies

Requires Phase 01 stabilization and benefits strongly from Phase 02 semantic oracle.

Benefits from Phase 03 proof receipts.

## Public Interface Changes

Potential commands:

```bash
jankurai contracts --check
jankurai db --check
jankurai generated --check
```

If separate commands are too much, implement as audit dimensions first.

Contract policy fields:

- source contract paths
- generated output paths
- generator command
- compatibility command
- breaking-change policy
- owner
- lane

DB policy fields:

- migration paths
- rollback policy
- destructive-change policy
- schema drift command
- query check command
- RLS policy requirement
- PII classification requirement

## Workstreams

### 1. Contract Source Detection

Implementation tasks:

- Detect OpenAPI, JSON Schema, Protobuf, and TypeSpec source paths.
- Detect generated clients under declared generated zones.
- Detect handwritten DTO/client drift in TypeScript and Python surfaces.
- Detect contract source changes without generated output or proof lane mapping.
- Add fixtures for REST and protobuf cases.

Acceptance:

- Public API surfaces without contracts are findings.
- Handwritten API client mirrors are findings.
- Generated clients are protected from hand edits.

### 2. Generated Zone Reproducibility

Implementation tasks:

- Enforce generated file metadata where policy applies:
  - generator
  - source
  - command
  - do-not-edit marker
- Add checksum or manifest support if practical.
- Add `doctor` or audit checks for generated zones with missing source command.
- **Slice 4 (done):** Audit rejects zone rows with empty `path` / `source` / `command` (same cap bucket as other generated-zone risk). Deeper doctor checksum work remains open.

Acceptance:

- Generated files tell agents exactly what source to edit and command to run.
- Missing generator metadata is actionable.

### 3. DB Migration Safety

Implementation tasks:

- Parse or scan migrations for destructive operations.
- Require rollback/backfill/lock-timeout/staged-deploy evidence for destructive migrations.
- Detect raw SQL outside allowed paths.
- Detect product invariants that appear app-only where DB constraints should exist, initially as advisory.
- Support SQLx check command detection where Rust/SQLx is used.

Acceptance:

- `DROP`, `TRUNCATE`, unbounded `DELETE`, and dangerous `ALTER` produce high-confidence findings unless exception evidence exists.
- Wrong-layer DB access points to adapters/db repair.
- DB proof lane appears in proof routing.

### 4. Event And Streaming Contracts

Implementation tasks:

- Enforce that event schemas live under `contracts/`.
- Enforce broker clients live under declared queue adapter paths.
- Require Kafka brownfield exceptions to include owner, expiry, reason/classification, and migration path.
- Add docs and fixtures for Kafka/Tansu/Iggy/Fluvio/NATS/Redis Streams markers.

Acceptance:

- `HLT-019-STREAMING-RUNTIME-DRIFT` remains stable and better evidenced.
- Streaming findings route to adapters/queue owners.

### 5. Contract And DB Report Integration

Implementation tasks:

- Add dimension evidence for contracts and DB truth.
- Add proof receipt links for contract and DB lanes.
- Add SARIF locations for contract and migration findings.
- Boundary manifest digest on **`repo-score`** JSON and evidence-index companion path (slice 2: audit ingest + **`boundaries_manifest_path`** when file exists).
- **Slice 3:** Markdown report (`render_markdown`) emits **`## Boundary manifest (ingested)`** when **`boundaries.artifact`** is present; GitHub step summary **`#### lane artifacts`** includes a **`boundaries`** line and appears when **only** boundaries ingest exists (truncated **`sha256:`** fingerprint in summary).

Acceptance:

- Contract and DB failures show in JSON, Markdown, SARIF, and repair queue.
- Findings include proof command and docs URL.

## Parallel MCP Breakdown

Strong parallel candidate:

- Agent A: contract source/generated detection. Owns contract modules and fixtures.
- Agent B: DB migration safety. Owns SQL/DB checks and fixtures.
- Agent C: streaming/event boundaries. Owns streaming checks and docs.
- Agent D: report integration. Owns rendering and schema updates.

Merge order:

1. Shared policy/schema fields.
2. Contract and DB analyzers.
3. Streaming analyzer.
4. Report integration and docs.

## Validation

Minimum:

```bash
cargo test -p jankurai
just fast
```

Fixture validation should include:

- contract source changed without generated output
- generated file without metadata
- destructive migration without evidence
- DB access from wrong layer
- streaming client outside adapter

## Risks

- Inferring app-only invariants can be noisy. Start advisory.
- Contract tooling varies; Jankurai should support generic source/generator metadata before choosing one generator.
- Migration safety needs careful exception policy to avoid blocking legitimate changes without a path forward.

## Handoff Notes

Leave:

- supported contract formats
- generated metadata requirements
- migration finding examples
- exception examples
- proof lane mappings
- known limitations

## Phase Status Receipt

- Phase status: partial contracts, DB, and generated boundaries; **doctor** validates boundary manifest; **audit** surfaces validated manifest digest on **`repo-score`** JSON; **Markdown audit output** and **GitHub step summaries** include **`## Boundary manifest (ingested)`** / lane-artifacts **`boundaries`** line when **`boundaries.artifact`** is present; **prove** may add **`boundaries_manifest_path`** on evidence index
- Operational handoff: [`tips/phases/logs/07-contracts-db-generated-boundaries.log`](logs/07-contracts-db-generated-boundaries.log) (append-only)
- Files changed (slice 1): `schemas/boundaries.schema.json`, `crates/jankurai/src/validation.rs`, `crates/jankurai/src/commands/doctor.rs`, `crates/jankurai/tests/boundaries_manifest_smoke.rs`, `crates/jankurai/tests/init_doctor.rs`, `crates/jankurai/tests/schema_contracts.rs`, `docs/moonshot.md`, `docs/testing.md`, `tips/phases/07-contracts-db-generated-boundaries.md`, `tips/phases/logs/README.txt`
- Files changed (slice 2): `crates/jankurai/src/model.rs`, `crates/jankurai/src/audit/boundaries_artifact.rs`, `crates/jankurai/src/audit/mod.rs`, `schemas/repo-score.schema.json`, `schemas/evidence-index.schema.json`, `crates/jankurai/src/commands/proof.rs`, `crates/jankurai/tests/boundaries_audit_ingest_smoke.rs`, `crates/jankurai/tests/schema_contracts.rs`, `crates/jankurai/tests/proof_surface_smoke.rs`, phase doc + log
- Files changed (slice 3): `crates/jankurai/src/render.rs`, `crates/jankurai/src/report/github.rs`, `crates/jankurai/tests/render_lane_artifacts_smoke.rs`, phase doc + log
- Files changed (slice 4): `crates/jankurai/src/audit/scan.rs`, `crates/jankurai/src/audit/mod.rs`, `crates/jankurai/src/audit/caps.rs`, `crates/jankurai/tests/generated_zones_manifest_smoke.rs`, `crates/jankurai/src/commands/migrate.rs`, `schemas/migration-report.schema.json`, `schemas/migration-plan.schema.json`, phase doc + log
- Schemas changed: `boundaries.schema.json` (slice 1); `repo-score.schema.json`, `evidence-index.schema.json` (slice 2); `migration-report.schema.json`, `migration-plan.schema.json` (slice 4: command/status envelope)
- Public interfaces changed: `ArtifactSchema::Boundaries`, `validation::validate_boundaries_toml_text`, doctor **`boundaries-manifest-schema`**; repo-score **`boundaries.artifact`**; evidence index **`boundaries_manifest_path`**
- Generated artifacts: none
- Routing maps changed: none
- Validation commands: `cargo test -p jankurai`, `just fast`
- Results: validation passed; DB enforcement remains partial
- Skipped validation: none
- Exceptions created: none
- Follow-up phases: 09 reference product platform, 10 reuse registry certified cells, 11 migration engine
