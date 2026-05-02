# Phase 13: Autonomous Repair And Optimization

Status: partial
Owner: agent
Last reviewed: 2026-05-02
Parallel MCP candidate: yes

## Objective

Add constrained autonomous repair and optimization after the proof, context, registry, migration, and certification systems are mature. This phase lets agents open bounded repair PRs, reduce token waste, remove dead code, fix drift, expire exceptions, and optimize performance under strict proof requirements.

This is intentionally the last phase. Autonomous repair without strong proof and permission boundaries would recreate vibe coding under a new name.

## Current State

Existing and planned prerequisites:

- Audit findings and repair queues exist.
- Phase 03 adds proof receipts.
- Phase 08 adds context packs, permission profiles, and repair packets.
- Phase 10 adds certified cells.
- Phase 11 adds migration slices.
- Phase 12 adds benchmark and certification evidence.

The implemented repair surface is dry-run only, with proof and permission gates keeping write execution out of the default path.

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
humanlint repair --plan target/humanlint/repair-plan.json
humanlint repair --auto-pr --max-risk low
humanlint optimize --budget latency:p95=80ms
humanlint reduce --tokens
humanlint refactor --proof-preserving
humanlint exceptions expire --repair
```

## Contract Slice

`repair-plan.schema.json` stays non-destructive. Minimum fields:

- `schema_version`, `source_report`, `generated_at`, `target_stack_id`
- `plan_mode` fixed to `dry-run`
- `planned_edits[]` with `path`, `operation`, `reason`
- `planned_commands`
- `proof_lanes`
- `rollback_guidance`
- `human_approval_requirements`
- `packets[]`

The current implementation starts with dry-run and PR-plan generation, not direct auto-merge.

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

### 2. Dry-Run Repair Planner

Implementation tasks:

- Convert repair packets into patch plans.
- Include expected file edits, commands, proof, and rollback.
- Do not write files in first iteration.
- Emit Markdown and JSON plans.

Acceptance:

- A human can approve or reject plan before edits.
- Plans are deterministic for the same input report.

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

### 4. Auto-PR Workflow

Implementation tasks:

- Generate branch, commit, PR body, and artifact links.
- Include report fingerprint, proof receipts, risk level, and residual risk.
- Keep PR draft by default.
- Require human review for medium/high risk.

Acceptance:

- Auto-PRs are transparent and auditable.
- PR body includes exact proof commands and artifacts.

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
cargo test -p humanlint
just fast
```

Repair dry-run smoke:

```bash
humanlint repair --plan target/humanlint/repair-plan.json --dry-run
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

## Phase Status Receipt

- Phase status: partial autonomous repair and optimization implementation slice
- Files changed: `schemas/repair-plan.schema.json`, `crates/humanlint/src/commands/repair_plan.rs`, `crates/humanlint/src/commands/context_pack.rs`, `crates/humanlint/src/commands/repair.rs`, `crates/humanlint/tests/command_surface_smoke.rs`, `docs/release-plan.md`, `docs/testing.md`, and `target/humanlint/phase-logs/13-autonomous-repair-optimization.md.log`
- Schemas changed: repair plan and repair packet support
- Public interfaces changed: `humanlint repair` with dry-run proof metadata
- Generated artifacts: repair plan JSON/Markdown outputs
- Routing maps changed: `agent/test-map.json`, `agent/owner-map.json`, `agent/proof-lanes.toml`
- Validation commands: `cargo test -p humanlint`, `just fast`
- Results: validation passed; optimization and bounded write execution remain future work
- Skipped validation: bounded patch execution, PR automation, and optimization loops remain gated for later expansion
- Exceptions created: dry-run repair only; write paths remain disabled until proof and permission gates mature
- Follow-up phases: none beyond the next implementation wave
