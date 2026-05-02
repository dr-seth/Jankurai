# Phase 05: UX Proof Platform

Status: partial
Owner: tools
Last reviewed: 2026-05-02
Parallel MCP candidate: yes

## Objective

Make rendered UX proof a first-class Humanlint lane. The goal is to replace repetitive pixel/layout QA with deterministic browser evidence wherever possible, while keeping human review for taste and ambiguous product judgment.

The exit state is a UX proof platform that can audit routes and Storybook stories, emit artifact-backed receipts, and feed Humanlint reports.

## Current State

Existing package:

- `packages/ux-qa/` is an npm workspace package named `@humanlint/ux-qa`.
- CLI supports `audit` and `storybook`.
- Checks include edge clearance, target size, interactive overlap, text clipping, button wrap, horizontal overflow, sticky obstruction, z-index ceiling, focus visible, form label, and nested scrollbar.
- CLI can emit screenshots, crops, and ARIA snapshots.
- Rust CLI has `humanlint ux` passthrough to `packages/ux-qa/dist/cli.js`.
- **`humanlint doctor`** validates **`agent/ux-qa.toml`** against **`schemas/ux-qa-policy.schema.json`** when that file exists (TOML parsed with the standard `toml` crate, then JSON-schema checked). **`ArtifactSchema::UxQaPolicy`** and **`validate_ux_qa_policy_toml_text`** in `crates/humanlint`; tests in `crates/humanlint/tests/ux_qa_policy_smoke.rs`. The `@humanlint/ux-qa` package still uses a line-oriented TOML subset for runtime—prefer simple tables and `[[routes]]` for parity.
- **`humanlint doctor`** validates **`target/humanlint/ux-qa.json`** against **`schemas/ux-qa.schema.json`** when that file exists (CLI output from `humanlint ux audit … --out …`). **`ArtifactSchema::UxQaReport`**; tests in `crates/humanlint/tests/ux_qa_report_smoke.rs`.
- **`humanlint audit`** (repo score JSON) ingests the same path when present and schema-valid: **`ux_qa.artifact`** holds a compact summary (`path`, `report_count`, **`worst_decision`** with ordering block > review > warn > pass, violation and summary counts). Invalid or missing files leave **`artifact`** omitted. Implementation in `crates/humanlint/src/audit/ux_artifact.rs`; tests in `crates/humanlint/tests/ux_qa_audit_ingest_smoke.rs`; **`schemas/repo-score.schema.json`** documents **`ux_qa`**. Score caps are unchanged.
- **`render_markdown`** and **GitHub step summary** (`report/github.rs`) print **`ux_qa.artifact`** when present; tests in `crates/humanlint/tests/render_lane_artifacts_smoke.rs`.
- Tests exist for geometry, artifacts, config, hit testing, selector, and Storybook discovery.

Gaps:

- Doctor’s TOML parser may accept constructs the UX CLI subset does not; keep policy files straightforward until parsers converge.
- UX decisions do not yet drive numeric audit score caps; evidence-index / proof receipts could still link digest fields more deeply than Markdown bullets alone.
- Accessibility tooling is not a first-class artifact yet.
- Visual baseline decisions are not standardized.
- Route/story matrix policy is still minimal.
- State coverage for loading, empty, error, success, permission-denied is not enforced.

## Dependencies

Requires Phase 01 docs/schema stability.

Benefits from Phase 03 proof receipts and evidence ledger.

## Public Interface Changes

UX CLI should converge on:

```bash
humanlint ux audit --config agent/ux-qa.toml --out target/humanlint/ux-qa.json
humanlint ux storybook --url http://localhost:6006 --config agent/ux-qa.toml
```

UX policy fields should include:

- routes
- storybook URL
- required viewports
- required states
- screenshot requirement
- ARIA snapshot requirement
- accessibility scan requirement
- visual baseline mode
- geometry thresholds
- artifact root
- merge decision thresholds

## Workstreams

### 1. UX Policy Schema

Implementation tasks:

- Expand `agent/ux-qa.toml` into a real policy file.
- Add schema for UX policy if missing or incomplete.
- Support route matrix and viewport matrix.
- Support per-route overrides.
- Support required states for critical UI surfaces.
- Keep defaults small enough for local execution.

Acceptance:

- Policy can express mobile/tablet/desktop viewports.
- Policy can express critical route IDs and Storybook story IDs.
- Policy can mark deterministic failures as blocking and visual diffs as review.

### 2. Accessibility Evidence

Implementation tasks:

- Integrate axe or an equivalent accessibility scanner through Playwright where practical.
- Emit accessibility JSON artifacts.
- Map findings to Humanlint UX/a11y rule IDs.
- Keep automated accessibility claims honest: automation catches common issues but does not replace all inclusive testing.

Acceptance:

- Critical route report can include accessibility artifact paths.
- Accessibility errors are distinguishable from geometry errors.
- Docs explain automated and manual boundaries.

### 3. Visual Baseline Decisions

Implementation tasks:

- Define baseline artifact paths under ignored output or approved baseline directories.
- Add visual diff metadata fields even if first implementation delegates actual diff to Playwright or external provider.
- Define merge decisions: pass, block, review.
- Add owner approval field for baseline updates.

Acceptance:

- A changed screenshot can be classified as deterministic pass/fail or owner-review visual diff.
- AI/VLM opinions remain advisory and cannot be the sole gate.

### 4. State Coverage

Implementation tasks:

- Define standard UI states: loading, empty, error, success, permission-denied.
- Add route/story metadata for state coverage.
- Detect missing critical states in configured surfaces.
- Encourage generated mocks/MSW for state generation, but do not require a specific mock tool yet.

Acceptance:

- UX report can say which required states were checked and which are missing.
- Missing state coverage becomes an audit finding for critical UI profiles.

### 5. Audit And Receipt Integration

Implementation tasks:

- Add UX report ingestion to Rust audit or proof receipt flow (JSON ingest shipped; Markdown + GitHub summary show compact **`ux_qa.artifact`** when present).
- Show compact UX summary in Markdown.
- Include artifacts in evidence ledger.
- Add findings for missing UX report on web-surface changes in strict modes.

Acceptance:

- `agent/repo-score.md` or future proof report can point to UX artifacts.
- A UI change without UX evidence is visible and actionable.

## Parallel MCP Breakdown

Strong parallel candidate:

- Agent A: policy schema and config parser. Owns `packages/ux-qa/src/config.ts`, schemas, config tests.
- Agent B: accessibility integration. Owns accessibility modules and tests.
- Agent C: visual baseline metadata. Owns artifact/receipt types and docs.
- Agent D: Rust audit/report integration. Owns Rust report ingestion and rendering.

Merge order:

1. Type/schema updates.
2. UX package feature work.
3. Rust ingestion.
4. Docs and examples.

## Validation

Minimum:

```bash
npm --workspace @humanlint/ux-qa run build
npm --workspace @humanlint/ux-qa run test
just fast
```

If Rust integration changes:

```bash
cargo test -p humanlint
```

Manual smoke with a running app or fixture:

```bash
humanlint ux audit --url http://localhost:3000 --out target/humanlint/ux-qa.json --screenshot --aria-snapshot
```

## Risks

- Visual diffs can be flaky without strict readiness contracts.
- Browser matrices can slow CI if run too often.
- Accessibility automation can create false confidence if docs imply complete coverage.

## Handoff Notes

Leave:

- UX policy schema
- sample route matrix
- sample report with artifacts
- list of deterministic blocking rules
- list of review-only visual rules
- validation artifacts and commands

## Phase Status Receipt

- Phase status: partial UX proof platform; **doctor validates `agent/ux-qa.toml`** (policy) and **`target/humanlint/ux-qa.json`** (report envelope) when those files exist; **audit** ingests validated `ux-qa.json` into **`repo-score` `ux_qa.artifact`**; human-facing **`repo-score.md`** and GitHub summaries surface the same ingest summary when present
- Operational handoff: [`tips/phases/logs/05-ux-proof-platform.log`](logs/05-ux-proof-platform.log) (append-only)
- Recent slice (report JSON): `schemas/ux-qa.schema.json` (`$defs` aligned to `packages/ux-qa/src/types.ts`), `ArtifactSchema::UxQaReport`, doctor `ux-qa-report-schema` path, `crates/humanlint/tests/ux_qa_report_smoke.rs`, `schema_contracts` assertions for `ux-qa.schema.json`
- Recent slice (audit ingest): `crates/humanlint/src/audit/ux_artifact.rs`, `UxQaReportArtifactSummary` + `UxQaReadiness.artifact` in `model.rs`, `schemas/repo-score.schema.json` `ux_qa` / `$defs`, `ux_qa_audit_ingest_smoke.rs`, `schema_contracts` repo-score `ux_qa` key
- Recent slice (Markdown / CI): `crates/humanlint/src/render.rs`, `crates/humanlint/src/report/github.rs`, `render_lane_artifacts_smoke.rs`
- Earlier slice (policy): `schemas/ux-qa-policy.schema.json`, `ArtifactSchema::UxQaPolicy`, `validate_ux_qa_policy_toml_text`, `crates/humanlint/tests/ux_qa_policy_smoke.rs`
- Schemas changed: `ux-qa.schema.json` — typed nested report, `schemaVersion` const `1.2.0`, rule/decision enums; `repo-score.schema.json` — **`ux_qa`** readiness + optional artifact summary
- Public interfaces changed (report): `ArtifactSchema::UxQaReport`, doctor checks `ux-qa-report-read` / `ux-qa-report-json` / `ux-qa-report-schema`; repo-score JSON **`ux_qa.artifact`** when ingest succeeds; Markdown / GitHub summary lines for ingest
- Generated artifacts: none
- Routing maps changed: none
- Validation commands: `cargo test -p humanlint`, `npm --workspace @humanlint/ux-qa run test`, `just fast`
- Results: see log file lines for SHA and outcomes
- Follow-up phases: 09 reference product platform, 12 benchmark certification and governance
