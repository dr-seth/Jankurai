# Jankurai

Title: **Jankurai: The Anti-Vibe Coding Standard**

Subtitle: **Continuous Repository Alignment for Proof-Carrying AI Pull Requests**

Thesis line: **Find the vibe. Prove the merge. Repair the repo.**

Paper edition: `2026.05-ed7`

Standard version: `0.7.0`

Schema version: `1.5.0`

Canonical source: `paper/jankurai.tex` plus `paper/tex/`

Rendered artifact: `paper/jankurai.pdf`

This Markdown file is an agent companion, not the canonical source. TeX remains the source of truth for the paper.

Naming policy: paper artifacts use the `jankurai.*` prefix. Do not create `main.md`, `main.tex`, or `main.pdf` anywhere in this repo.

## Executive Abstract

AI-assisted coding makes plausible code cheap and vibe-code drift expensive. Traditional technical debt becomes evidence debt when a repository cannot prove ownership, generated boundaries, changed behavior, or refactoring safety. Jankurai is a versioned repository conformance standard for finding and reducing vibe artifacts: ownerless paths, unmapped proof, hand-edited generated zones, stale contracts, false-green tests, missing security evidence, overbroad agent permissions, unproven UI changes, stale waivers, and unreceipted review claims.

The repository is the alignment layer. In this paper, "proof" means repository-local evidence receipts, not formal proof of full program semantics. Jankurai maps changed paths to bounded authority, proof lanes, evidence receipts, repair queues, and expiring waivers. The central artifact is the merge witness: a versioned binding among changed paths, owner routes, required proof receipts, observed evidence, missing-evidence decisions, artifact digests, tool identity, and commit identity.

Jankurai Core is stack-neutral. Rust/TypeScript/PostgreSQL is a non-normative reference profile, not the standard. Go, .NET, JVM, TypeScript-heavy, Rails/Python, and Elixir profiles can conform when they emit equivalent owner routes, proof receipts, generated-zone evidence, security/UX evidence, and merge witnesses. This reference workspace remains Rust-first: agents must not add Python for product truth, proof lanes, repo tools, product services, authorization, database writes, or general backend glue. Python is allowed only for rare advanced ML/data library work with a dated exception under `python/ai-service`. Agent-first repository design means code and policy are shaped so agents can find owners, avoid generated zones, run one proof lane, receive stable failures, repair narrow scope, and leave receipts. Scores are posture signals, not merge approval.

The May 5, 2026 public-repository advisory scan is the paper's field-evidence section. Jankurai 0.7.0 scanned 30 public GitHub repositories, succeeded on all 30, and observed a top score of 48, average score of 33.4, 15,391 total findings, and 15,017 hard findings. The scan is framed as repair-oriented posture evidence, not certification, defect attribution, or an incident study.

## Section Map

1. From Language Chaos to Verified Merge
2. Running Example: Checkout PR
3. Definitions and Threat Model
4. Jankurai Core Standard and Conformance
5. Vibe-Artifact Taxonomy and Stable Rule IDs
6. Evaluation and Conformance Evidence
7. Public Repository Scoring in the Wild
8. Agent Repository Controls and Tool Adapters
9. Continuous Proof: From Changed Paths to Merge Witness
10. Rendered UX and Browser-Step QA
11. Security, Supply Chain, and Permissions
12. Waivers, Observability, and Repair Receipts
13. Migration, Versioning, and Governance
14. Languages as Proof-Cost Compression
15. Technical Promise Versus Standard Gravity
16. Non-Normative Reference Profile Score
17. Reference Profile Comparison
18. Reference Architecture Profile
19. Related Work
20. Limitations and Research Agenda
21. Conclusion

Appendices:

- Rule IDs and Conformance Evidence
- Versioned Artifact Manifest
- Waiver and Repair Templates
- Reference-Profile File Tree Diagrams
- Golden First-Hour Command Path
- Public Repository Score Details

## Core Interfaces

- Decision enum: `pass`, `review`, `block`, `ratchet_fail`, `release_fail`.
- Machine schema field: `schema_version`.
- Schema bundle: prose for the schema collection identified by `schema_version`.
- Emitted report/profile field: `target_stack_id`.
- Current manifest profile label: `target_stack`.
- `paper_edition`: provenance, not a required core conformance field.

Compact conformance claim:

```json
{
  "standard_version": "0.7.0",
  "auditor_version": "0.7.0",
  "schema_version": "1.5.0",
  "claimed_level": "HL3",
  "current_commit": "9f3a1c4",
  "decision": "block",
  "required_receipts": ["web", "contract", "security"],
  "missing_evidence": ["rendered_ux_receipt"],
  "witness_path": "target/jankurai/merge-witness.json"
}
```

## Conformance Evidence

The current seed suite under `conformance/` has 10 fixture directories and 12 expected JSON files. `rtk just conformance` currently validates fixture inventory and expected JSON presence; observed per-fixture witness comparison is future work.

- `hl3-pass-minimal` expects `pass`.
- Nine fail fixtures expect `block`.
- Primary rule examples: `HLT-002`, `HLT-003`, `HLT-004`, `HLT-010`, `HLT-012`, `HLT-013`, `HLT-021`, `HLT-022`, `HLT-023`.
- Validation command: `rtk just conformance`.

## Public Repository Field Scan

- Source data: `paper/data/public-repo-scores-20260505T184426Z.json`.
- Receipt: `paper/data/public-repo-scores-20260505T184426Z.json.sha256`.
- Generated tables: `paper/tex/generated/public_repo_score_tables.tex`.
- Regeneration command: `cargo run -p jankurai -- paper public-repo-scores --source paper/data/public-repo-scores-20260505T184426Z.json --out paper/tex/generated/public_repo_score_tables.tex`.
- Note: this paper-table helper is Rust and is not a product/runtime dependency.
- Scope: 30 public GitHub repositories, 30 successful scans, 0 failed scans.
- Aggregate posture: min 14, max 48, average 33.4, upper-middle score 34, hard finding share 97.6%.

## Key Rules

- `HLT-001-DEAD-MARKER`: future-hostile markers in product/runtime code need repair or a dated waiver.
- `HLT-002-GENERATED-MUTATION`: generated outputs are changed through source contracts and regeneration.
- `HLT-003-OWNERLESS-PATH`: changed paths need an owner-map entry.
- `HLT-004-UNMAPPED-PROOF`: changed paths need a mapped proof lane.
- `HLT-005-PYTHON-PRODUCT-TRUTH`: Python must not silently own durable product truth, and this workspace permits Python only as a rare dated advanced-ML/data exception.
- `HLT-006-DIRECT-DB-WRONG-LAYER`: DB access belongs in declared durable-truth or adapter boundaries.
- `HLT-007-HANDWRITTEN-CONTRACT`: public contracts should generate clients/stubs.
- `HLT-008-FALSE-GREEN-RISK`: tests must prove the changed behavior, not just pass nearby.
- `HLT-009-GENERATED-SECURITY`: security-sensitive generated code needs security proof.
- `HLT-010-SECRET-SPRAWL`: secret-like values, env dumps, and transcript leaks are hard failures.
- `HLT-011-PROMPT-INJECTION`: untrusted context cannot override trusted policy.
- `HLT-012-OVERBROAD-AGENCY`: agent permissions must match the proof lane.
- `HLT-013-RENDERED-UX-GAP`: web surfaces need artifact-backed rendered UX evidence.
- `HLT-014-A11Y-GAP`: changed UI surfaces need accessibility evidence.
- `HLT-015-CONTEXT-SETUP-GAP`: setup and context routing must be deterministic.
- `HLT-016-SUPPLY-CHAIN-DRIFT`: dependency and provenance changes need review evidence.
- `HLT-017-OPAQUE-OBSERVABILITY`: boundary failures need repairable telemetry.
- `HLT-018-PERF-CONCURRENCY-DRIFT`: performance and concurrency risk needs proof.
- `HLT-019-STREAMING-RUNTIME-DRIFT`: broker clients and stack identity must stay behind adapter boundaries.
- `HLT-020-CI-HARDENING-GAP`: CI workflow permissions, action pinning, and proof posture gaps need repair.
- `HLT-021-DESTRUCTIVE-MIGRATION`: destructive SQL under migration paths needs documented safety evidence.
- `HLT-022-AUTHZ-ISOLATION-GAP`: authorization and tenant/data isolation need negative proof.
- `HLT-023-INPUT-BOUNDARY-GAP`: unsafe input boundaries and dynamic sinks need exploit-focused proof.
- `HLT-024-AGENT-TOOL-SUPPLY-GAP`: agent tools, MCP servers, hooks, and rule files need trust evidence.
- `HLT-025-RELEASE-READINESS-GAP`: release claims need backup, monitoring, rollback, security, and abuse-control evidence.
- `HLT-026-COST-BUDGET-GAP`: paid or unbounded operations need budgets, quotas, and stop conditions.
- `HLT-027-HUMAN-REVIEW-EVIDENCE-GAP`: review and proof claims need receipts and replayable commands.

## Metrics

- Vibe artifact density
- Proof coverage ratio
- Owner route coverage
- Generated-zone drift count
- Repair half-life
- Context efficiency
- Witness completeness
- Alignment drift
- Proof-selection accuracy

## Validation

```bash
just versions
just conformance
just paper
just check
```

The audit lane is:

```bash
cargo run -p jankurai -- . --json agent/repo-score.json --md agent/repo-score.md
```

The paper lane is:

```bash
latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=paper paper/jankurai.tex
```
