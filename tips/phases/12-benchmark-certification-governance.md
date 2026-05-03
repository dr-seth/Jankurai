# Phase 12: Benchmark Certification And Governance

Status: complete
Owner: standard
Last reviewed: 2026-05-03
Parallel MCP candidate: yes

## Objective

Prove the Jankurai thesis publicly and make conformance meaningful. This phase builds benchmark corpora, certification artifacts, badges, release attestations, rule governance, and organization-level reporting.

The exit state is that "Built with Jankurai" has evidence behind it.

## Current State

Completed pieces:

- `jankurai bench` emits a schema-valid `BenchmarkReport` with a bundled smoke suite built from `examples/legacy-node-api/` and `examples/perfect-web-api-db/`.
- `jankurai certify` emits a schema-valid `Certification` artifact tied to the live repo score when present and otherwise falls back to an explicit missing-score state.
- `jankurai govern` emits a schema-valid `GovernancePolicy` from the standard manifest.
- The outputs validate through `ArtifactSchema` before write or stdout emission.
- No external signing service, hosted dashboard, or public badge publishing was added.

## Dependencies

Requires phases 01 through 11 to produce enough real surfaces to benchmark.

## Public Interface Changes

Implemented command surface:

```bash
jankurai bench . --out target/jankurai/p12-benchmark-report.json --md target/jankurai/p12-benchmark-report.md
jankurai certify . --out target/jankurai/p12-certification.json --md target/jankurai/p12-certification.md
jankurai govern . --out target/jankurai/p12-governance-policy.json --md target/jankurai/p12-governance-policy.md
```

## Contract Slice

`benchmark-suite.schema.json` defines the bundled benchmark corpus. Minimum fields:

- `schema_version`, `suite_id`, `purpose`
- `fixtures[]` with `fixture_id`, `path`, `kind`, `expected_findings`, `expected_score_range`
- `tasks[]` with `task_id`, `description`, `fixture_ids`, `commands`, `expected_metrics`

`benchmark-report.schema.json` defines run output. Minimum fields:

- `schema_version`, `generated_at`, `suite_id`, `repo`, `target_stack_id`
- `results[]` with `task_id`, `fixture_id`, `status`, `metrics`, `evidence`
- `summary`

`certification.schema.json` defines release evidence. Minimum fields:

- `schema_version`, `generated_at`, `repo`
- `standard_version`, `auditor_version`, `paper_edition`, `target_stack_id`
- `score`, `conformance_level`, `caps`
- `findings_summary`
- `proof_receipt_index`, `security_receipt_index`, `ux_receipt_index`, `contract_db_receipt_index`
- `exceptions`
- `provenance`

`governance-policy.schema.json` defines ratchet policy. Minimum fields:

- `schema_version`, `standard_version`, `effective_at`
- `minimum_score`, `fail_on`, `advisory_on`, `update_channel`
- `rule_change_policy`, `deprecation_policy`, `exception_policy`, `security_advisory_policy`
- `rfc_path`

Certification artifact fields:

- repo
- standard version
- auditor version
- schema version
- paper edition
- target stack ID
- score
- conformance level
- caps
- findings summary
- proof receipt index
- security receipt index
- UX receipt index
- contract/DB receipt index
- exceptions
- provenance attestation only; no external signature

## Validation

Validated:

```bash
cargo test -p jankurai
just fast
just score
```

Phase proof:

```bash
cargo run -p jankurai -- lane . \
  --changed crates/jankurai/src/commands/bench.rs \
  --changed crates/jankurai/src/commands/certify.rs \
  --changed crates/jankurai/src/commands/govern.rs \
  --changed schemas/certification.schema.json \
  --out target/jankurai/p12-benchmark-certification-lane.json \
  --md target/jankurai/p12-benchmark-certification-lane.md
```

## Closeout

Artifacts:

- `target/jankurai/p12-benchmark-report.json`
- `target/jankurai/p12-benchmark-report.md`
- `target/jankurai/p12-certification.json`
- `target/jankurai/p12-certification.md`
- `target/jankurai/p12-governance-policy.json`
- `target/jankurai/p12-governance-policy.md`
- `target/jankurai/p12-benchmark-certification-lane.json`
- `target/jankurai/p12-benchmark-certification-lane.md`
- `target/jankurai/fast-score.json`
- `target/jankurai/fast-score.md`
- `agent/repo-score.json`
- `agent/repo-score.md`

Validated:

- `cargo test -p jankurai`
- `cargo run -p jankurai -- lane . --changed crates/jankurai/src/commands/bench.rs --changed crates/jankurai/src/commands/certify.rs --changed crates/jankurai/src/commands/govern.rs --changed schemas/certification.schema.json --out target/jankurai/p12-benchmark-certification-lane.json --md target/jankurai/p12-benchmark-certification-lane.md`
- `just fast`
- `just score`
