# Phase 13: Autonomous Repair And Optimization

Status: partial
Owner: agent
Last reviewed: 2026-05-03
Parallel MCP candidate: yes

## Objective

Add constrained autonomous repair and optimization after the proof, context, registry, migration, and certification systems are mature. The current slice is deliberately narrower: risk-gated dry-run repair planning plus fixture-only bounded patch execution receipts.

This is intentionally the last phase. Autonomous repair without strong proof and permission boundaries would recreate vibe coding under a new name.

## Current State

Existing and planned prerequisites:

- Audit findings and repair queues exist.
- Phase 03 adds proof receipts.
- Phase 08 adds context packs, permission profiles, and repair packets.
- Phase 10 adds certified cells.
- Phase 11 adds migration slices.
- Phase 12 adds benchmark and certification evidence.

The implemented repair surface is dry-run by default with an explicit fixture-only apply mode. Repair packets and repair plans now carry explicit eligibility, risk, planned edits, planned proof commands, rollback guidance, human approval requirements, and structured patch fields for fixture plans. Repair runs can evaluate whether an auto-PR request would be blocked or eligible, emit a draft-only PR evidence package, and fixture-marked repositories can execute bounded `append-text`, `replace-exact`, and `create-file` edits. The command still does not write real projects, create branches, commit, open PRs, or auto-merge.

## Dependencies

Hard dependencies:

- Phase 03 proof router
- Phase 08 repair packets and permission profiles
- Phase 10 cell registry
- Phase 11 migration engine
- Phase 12 certification and benchmark governance

## Public Interface Changes

Implemented command surface:

```bash
jankurai repair-plan . --from target/jankurai/repo-score.json --out target/jankurai/repair-plan.json --md target/jankurai/repair-plan.md
jankurai repair . --plan target/jankurai/repair-plan.json --dry-run --out target/jankurai/repair-run.json --md target/jankurai/repair-run.md
jankurai repair . --plan target/jankurai/repair-plan.json --dry-run --auto-pr --max-risk low
jankurai repair . --plan target/jankurai/repair-plan.json --dry-run --auto-pr --max-risk medium --pr-draft-out target/jankurai/repair-pr-draft.json --pr-draft-md target/jankurai/repair-pr-draft.md
jankurai repair target/jankurai/p13-fixture-repo --plan target/jankurai/p13-fixture-repo/target/jankurai/repair-plan.json --fixture-apply --max-risk medium --out target/jankurai/p13-fixture-repair-run.json --md target/jankurai/p13-fixture-repair-run.md
```

Deferred command surface:

- No real repository patch execution yet; bounded patch execution is fixture-only and requires `agent/repair-fixture.toml` with `fixture = true`.
- No real auto-PR creation yet; draft-package evidence is emitted behind `--auto-pr`.
- No `optimize`, `reduce`, or `refactor` commands yet.
- No `exceptions expire` loop yet.

## Contract Slice

`repair-plan.schema.json` stays non-destructive. Minimum fields:

- `schema_version`, `source_report`, `generated_at`, `target_stack_id`
- `plan_mode` fixed to `dry-run`
- `planned_edits[]` with `path`, `operation`, `reason`, `finding_fingerprint`, `rule_id`, `apply_strategy`, and optional patch text fields
- packet `repair_eligibility`, `risk_level`, and `eligibility_reason`
- `planned_commands`
- `proof_lanes`
- `rollback_guidance`
- `human_approval_requirements`
- `packets[]`

`repair-run.schema.json` records execution mode, repair execution status, auto-PR dry-run eligibility, optional auto-PR draft summary, max risk, blocked packets, risk summary, proof lanes, applied edits, skipped edits, files written, optional proof evidence index, and notes.

`repair-pr-draft.schema.json` records the draft-only PR evidence package with branch name, titles, planned paths, eligible and blocked packets, proof lanes, artifact links, residual risk, and mutation flags.

The current implementation supports dry-run planning, dry-run auto-PR eligibility reporting, and fixture-only patch execution. It does not support real repository patch execution, branch creation, PR creation, or auto-merge.

## Safety Principles

Autonomous repair may only act when:

- finding has stable fingerprint
- rule has repair eligibility
- allowed paths are explicit
- forbidden paths are explicit
- generated zones are protected
- permission profile allows edits
- proof lanes are executable
- rollback or revert plan exists
- human review requirement is clear
- max risk threshold is not exceeded

Autonomous repair must not:

- perform destructive migrations
- rotate secrets
- change production infrastructure credentials
- edit generated files by hand
- broaden agent permissions
- rewrite architecture broadly
- merge without proof
- hide failed tests
- create undocumented exceptions

## Workstreams

### 1. Repair Eligibility

Implementation tasks:

- Add eligibility metadata to rules:
  - auto-safe
  - agent-assisted
  - human-required
  - never-auto
- Define risk levels.
- Add tests that high-risk rules cannot auto-run.

Acceptance:

- Every auto-repairable rule declares why it is safe.
- Destructive or ambiguous work is human-required.
- Secret sprawl is never-auto and critical.

### 2. Dry-Run Repair Planner

Implementation tasks:

- Convert repair packets into patch plans.
- Include expected file edits, commands, proof, and rollback.
- Do not write files in first iteration.
- Emit Markdown and JSON plans.

Acceptance:

- A human can approve or reject plan before edits.
- Plans are deterministic for the same input report.
- Plans validate against `schemas/repair-plan.schema.json`.

### 3. Bounded Patch Execution

Implementation tasks:

- Apply edits only within allowed paths.
- Refuse generated zones unless running declared generator.
- Refuse changes outside task scope.
- Run proof lanes after patch.
- Emit repair receipt.

Acceptance:

- Patch cannot escape allowed paths.
- Failed proof stops repair and records evidence.

Status: partial. Fixture-only execution is implemented behind `--fixture-apply`; real repository patch execution remains deferred.

### 4. Auto-PR Workflow

Implementation tasks:

- Generate branch, commit, PR body, and artifact links.
- Include report fingerprint, proof receipts, risk level, and residual risk.
- Keep PR draft by default.
- Require human review for medium/high risk.

Acceptance:

- Auto-PR draft packages are transparent and auditable.
- Draft body includes exact proof lanes, artifact links, and residual risk.

Status: partial. Current `--auto-pr` emits a draft-only evidence package, while real branch, commit, and GitHub PR creation remain deferred.

### 5. Optimization Commands

Implementation tasks:

- Token reduction:
  - shorten root docs
  - move durable detail into routed docs
  - update context maps
  - remove duplicate agent instructions
- Performance:
  - detect budget regressions
  - suggest targeted fixes
  - require benchmark proof
- Dependency cleanup:
  - identify unused dependencies
  - require build/test proof
- Dead code:
  - identify orphan code
  - require reachability and tests

Acceptance:

- Optimization never removes behavior without proof.
- Token reduction reports before/after context size.

Status: deferred.

### 6. Exception Expiry Loop

Implementation tasks:

- Detect expired exceptions.
- Generate repair plans:
  - remove exception by fixing violation
  - renew with owner and justification
  - escalate to human
- Add dashboard/report summary.

Acceptance:

- Expired exceptions cannot silently persist.
- Repair options are explicit.

Status: deferred.

## Parallel MCP Breakdown

Partial parallel candidate:

- Agent A: eligibility and risk model.
- Agent B: dry-run planner.
- Agent C: patch execution sandbox.
- Agent D: auto-PR integration.
- Agent E: optimization subcommands.
- Agent F: exception expiry loop.

Do not parallelize patch execution and permission model changes until the permission profile schema is locked.

Merge order:

1. Eligibility/risk model.
2. Dry-run planner.
3. Patch execution.
4. Auto-PR.
5. Optimization and exception loops.

## Validation

Minimum:

```bash
cargo test -p jankurai
just fast
```

Repair dry-run smoke:

```bash
jankurai repair . --plan target/jankurai/repair-plan.json --dry-run
```

Patch execution must use fixture repos before touching real projects.

## Risks

- Autonomous repair can become vibe coding if proof is weak.
- Agents can overfit to tests and miss product intent.
- Auto-PRs can spam maintainers if repair queue prioritization is poor.
- Optimization can remove useful context if token budget is valued over clarity.

## Handoff Notes

Leave:

- eligibility matrix
- risk policy
- dry-run examples
- fixture repair results
- proof receipts
- auto-PR template
- known never-auto rules
- residual risk that patch execution, real auto-PRs, optimization commands, and exception expiry remain deferred

## Phase Status Receipt

- Phase status: partial autonomous repair and optimization implementation slice
- Files changed in this slice: `crates/jankurai/src/audit/rules.rs`, `crates/jankurai/src/commands/repair_plan.rs`, `crates/jankurai/src/commands/repair.rs`, `crates/jankurai/src/validation.rs`, `schemas/repair-plan.schema.json`, `schemas/repair-packet.schema.json`, `schemas/repair-run.schema.json`, focused repair/schema tests, and this phase receipt.
- Schemas changed: repair packet metadata, dry-run repair plan fields, and repair-run receipts.
- Public interfaces changed: `jankurai repair-plan` emits dry-run plans; `jankurai repair --dry-run` emits schema-valid repair-run JSON/Markdown; `--auto-pr` reports dry-run eligibility only.
- Generated artifacts: repair plan JSON/Markdown, repair-run JSON/Markdown, and proof lane outputs under `target/jankurai/`.
- Routing maps changed: none in this slice.
- Deferred: bounded patch execution, real auto-PR creation, optimization commands, and exception expiry.
- Results: validation passed; optimization and bounded write execution remain future work
- Skipped validation: bounded patch execution, PR automation, and optimization loops remain gated for later expansion
- Exceptions created: dry-run repair only; write paths remain disabled until proof and permission gates mature
- Follow-up phases: none beyond the next implementation wave
