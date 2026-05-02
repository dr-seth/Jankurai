# Phase 12: Benchmark Certification And Governance

Status: partial
Owner: standard
Last reviewed: 2026-05-02
Parallel MCP candidate: yes

## Objective

Prove the Jankurai thesis publicly and make conformance meaningful. This phase builds benchmark corpora, certification artifacts, badges, release attestations, rule governance, and organization-level reporting.

The exit state is that "Built with Jankurai" has evidence behind it.

## Current State

Existing pieces:

- Paper and standard are versioned.
- Reports include standard/auditor/schema/paper/target-stack bindings.
- `docs/release-plan.md` defines release lines, channels, CI adoption, benchmark pack, and stable compliance targets.
- The `jankurai bench`, `jankurai certify`, and `jankurai govern` commands now emit evidence-bound plans and governance artifacts from live repo data.

## Dependencies

Requires phases 01 through 11 to produce enough real surfaces to benchmark.

## Public Interface Changes

Implemented command surface:

```bash
jankurai certify --out target/jankurai/certification.json
jankurai bench run --suite agent-success
jankurai bench report
jankurai govern check
```

## Contract Slice

`benchmark-suite.schema.json` should define the benchmark corpus. Minimum fields:

- `schema_version`, `suite_id`, `purpose`
- `fixtures[]` with `fixture_id`, `path`, `kind`, `expected_findings`, `expected_score_range`
- `tasks[]` with `task_id`, `description`, `fixture_ids`, `commands`, `expected_metrics`

`benchmark-report.schema.json` should define run output. Minimum fields:

- `schema_version`, `generated_at`, `suite_id`, `repo`, `target_stack_id`
- `results[]` with `task_id`, `fixture_id`, `status`, `metrics`, `evidence`
- `summary`

`certification.schema.json` should define release evidence. Minimum fields:

- `schema_version`, `generated_at`, `repo`
- `standard_version`, `auditor_version`, `paper_edition`, `target_stack_id`
- `score`, `conformance_level`, `caps`
- `findings_summary`
- `proof_receipt_index`, `security_receipt_index`, `ux_receipt_index`, `contract_db_receipt_index`
- `exceptions`
- `provenance`

`governance-policy.schema.json` should define ratchet policy. Minimum fields:

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
- signature/provenance if available

## Workstreams

### 1. Benchmark Corpus

Implementation tasks:

- Define benchmark suite structure.
- Add fixture repos or external fixture references.
- Include bad and good examples:
  - vibe-coded React app
  - legacy Node API
  - overgrown Python service
  - DTO drift repo
  - silent fallback repo
  - unsafe migration repo
  - UX regression fixture
  - generated-contract golden repo
  - Jankurai-native golden repo
- Keep fixture size controlled.

Acceptance:

- Each fixture has expected findings and score range.
- Benchmark docs explain methodology and limitations.

### 2. Agent Benchmark Metrics

Implementation tasks:

- Define tasks:
  - add auth-protected page
  - add API endpoint and generated client
  - add DB migration safely
  - fix visual overflow
  - fix prompt injection
  - add background job
  - migrate one module slice
- Capture metrics:
  - token usage where available
  - wrong-file edit rate
  - time to first correct patch
  - commands run
  - proof correctness
  - regression rate
  - human review burden
- Provide manual recording fallback when agent APIs do not expose all metrics.

Acceptance:

- Benchmark can compare baseline repo and Jankurai-native repo.
- Results are reproducible enough for public claims.

### 3. Certification And Badges

Implementation tasks:

- Define conformance evidence requirements.
- Add badge JSON or SVG generation.
- Add signed or attestable certification output where practical.
- Include score, standard version, and conformance level.
- Add failure reasons for non-certification.

Acceptance:

- Badge cannot claim conformance without evidence.
- Certification is tied to versions and report fingerprint.

### 4. Governance Model

Implementation tasks:

- Define rule change process.
- Define versioning policy for standard, auditor, schema, paper, rule packs, templates, cells.
- Define deprecation policy.
- Define exception economics.
- Define security advisory process.
- Define public RFC path if project governance expands.

Acceptance:

- Breaking rule changes require version bump and migration notes.
- New advisory checks can ship without breaking adopters.
- Exceptions remain time-boxed and visible.

### 5. Organization Control Plane

Implementation tasks:

- Define org-level inventory schema:
  - repos
  - scores
  - exceptions
  - dependency risks
  - proof receipts
  - security posture
  - UX evidence
  - migration queues
  - reusable cell adoption
- CLI remains source of truth; dashboards consume artifacts.

Acceptance:

- Executive reporting can be produced from artifact files.
- Dashboard is optional, not required for local use.

## Parallel MCP Breakdown

Strong parallel candidate:

- Agent A: benchmark corpus and expected results.
- Agent B: metric capture and bench runner.
- Agent C: certification artifact and badge.
- Agent D: governance docs.
- Agent E: org-level schema/reporting.

Merge order:

1. Certification and benchmark schemas.
2. Corpus and runner.
3. Governance docs.
4. Badges and org reporting.

## Validation

Minimum:

```bash
cargo test -p jankurai
just fast
```

Benchmark smoke:

```bash
jankurai bench run --suite smoke
jankurai certify --out target/jankurai/certification.json
```

Use equivalent command names if implementation chooses different names.

## Risks

- Benchmarks can be accused of being cherry-picked. Include limitations and raw evidence.
- Badges can become marketing if not tied to report fingerprints.
- Governance can slow the project if overbuilt before adoption.

## Handoff Notes

Leave:

- benchmark methodology
- fixture inventory
- certification schema
- badge semantics
- governance policy
- public claim language approved by evidence

## Phase Status Receipt

- Phase status: partial benchmark certification and governance implementation slice
- Files changed: `schemas/benchmark-suite.schema.json`, `schemas/benchmark-report.schema.json`, `schemas/certification.schema.json`, `schemas/governance-policy.schema.json`, `crates/jankurai/src/commands/bench.rs`, `crates/jankurai/src/commands/certify.rs`, `crates/jankurai/src/commands/govern.rs`, and `target/jankurai/phase-logs/12-benchmark-certification-governance.md.log`
- Schemas changed: benchmark suite, benchmark report, certification, governance policy
- Public interfaces changed: `jankurai bench`, `jankurai certify`, and `jankurai govern`
- Generated artifacts: benchmark plan and certification plan outputs
- Routing maps changed: `agent/test-map.json`, `agent/owner-map.json`, `agent/proof-lanes.toml`
- Validation commands: `cargo test -p jankurai`, `just fast`
- Results: validation passed; benchmark and governance surfaces remain planner-only
- Skipped validation: public badge and org reporting workflows remain bounded for later expansion
- Exceptions created: certification remains evidence-bound, not an attestation issuer
- Follow-up phases: 13 autonomous repair and optimization
