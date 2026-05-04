# Trustworthy Merge

Public banner: **Humans Were the Bug**

Subtitle: **From Vibe Coding to Agent-Native Engineering**

Paper edition: `2026.05-ed4`

Standard version: `0.5.0`

Canonical source: `paper/jankurai.tex` plus `paper/tex/`

Rendered artifact: `paper/jankurai.pdf`

This Markdown file is an agent companion, not the canonical source. TeX remains the source of truth for the paper.

Naming policy: paper artifacts use the `jankurai.*` prefix. Do not create `main.md`, `main.tex`, or `main.pdf` anywhere in this repo.

## Executive Abstract

AI coding moves the bottleneck from typing plausible code to trustworthy merge. Agent-native engineering treats the repository as a verification interface for all code entering a governed repo, whether the first draft came from a person or a model. Ownership, proof lanes, generated zones, security gates, proof ledgers, repair queues, receipts, and versioned artifacts must be machine-readable.

`Jankurai` is the control plane for repositories that claim conformance. It defines progressive install levels, conformance levels, stable rule IDs, continuous local/PR/CI audit, merge witnesses, rolling score, proof verification, security evidence, CI modes, version bindings, and governed repair. The default reference profile remains Rust core, TypeScript/React/Vite product surface, PostgreSQL durable truth, generated contracts, and bounded Python for AI/data service work.

## Section Map

1. New Bottleneck: Trustworthy Merge
2. Definitions and Threat Model
3. Languages as Bottleneck Compression
4. Technical Promise Versus Standard Gravity
5. Vibe-Coding Fault Taxonomy
6. Jankurai Standard and Conformance
7. A 100-Point Stack Rubric
8. Stack Ranking, Sensitivity, and Exceptions
9. Default Winner Architecture
10. Agent Repository Controls and Tool Adapters
11. Continuous Proof, CI, and Test Explosion
12. Rendered UX and Browser-Step QA
13. Security, Supply Chain, and Permissions
14. Exceptions, Observability, and Repair Receipts
15. Migration, Versioning, and Governance
16. Limitations and Research Agenda
17. Conclusion

Appendices:

- Rule IDs and Conformance Evidence
- Versioned Artifact Manifest
- Exception and Repair Templates
- Canonical File Tree Diagrams
- Command Examples for Adoption and Proof

## Core Claims

- The scarce act is trustworthy merge, not first-draft code generation.
- Public rule: no proof, no merge; no receipt, no trust.
- A repository claiming jankurai conformance must expose auditable ownership, proof routing, generated-zone policy, version metadata, proof ledgers, and repair evidence.
- Human-authored code and agent-authored code should meet the same merge-time controls in a governed repo.
- Adoption is progressive: `agents -> score -> ci -> full -> ratchet`. Minimal agent hooks shape tool behavior before a repository claims conformance.
- The operating loop is changed path -> owner/test route -> context pack -> proof plan -> receipts/evidence index -> proof verification -> audit report -> merge witness -> repair queue.
- Stack choice matters after the control plane exists. The default winner is Rust core, TypeScript/React/Vite, PostgreSQL, generated contracts, and bounded Python.
- Hard caps and score weights are versioned policy, not final empirical truth.
- The TLR pie chart is computed from the visible taxonomy RPN rows; it is a policy-priority model, not an incident-frequency chart.
- Rendered UX QA is first-class for critical UI flows: Storybook states, screenshots, ARIA snapshots, accessibility, CLS, visual review, design-token evidence, generated mocks, and DOM geometry rules should catch routine layout defects before human taste review.
- Streaming infrastructure is workload-specific. Kafka is valid brownfield infrastructure behind generated event contracts and Rust adapters; Tansu is the leading Kafka-compatible Rust candidate to evaluate; Apache Iggy and Fluvio are Rust-native greenfield candidates, not drop-ins.

## Key Rules

- `HLT-001-DEAD-MARKER`: future-hostile markers in product/runtime code need repair or a dated exception.
- `HLT-002-GENERATED-MUTATION`: generated outputs are changed through source contracts and regeneration.
- `HLT-003-OWNERLESS-PATH`: changed paths need an owner-map entry.
- `HLT-004-UNMAPPED-PROOF`: changed paths need a mapped proof lane.
- `HLT-005-PYTHON-PRODUCT-TRUTH`: Python stays boxed to AI/data service or tooling.
- `HLT-006-DIRECT-DB-WRONG-LAYER`: DB access belongs in adapters or migrations.
- `HLT-007-HANDWRITTEN-CONTRACT`: public contracts should generate clients/stubs.
- `HLT-008-FALSE-GREEN-RISK`: tests must prove the changed behavior, not just pass nearby.
- `HLT-009-GENERATED-SECURITY`: security-sensitive generated code needs security proof.
- `HLT-010-SECRET-SPRAWL`: secret-like values, env dumps, and transcript leaks are hard failures.
- `HLT-011-PROMPT-INJECTION`: untrusted context cannot override trusted policy.
- `HLT-012-OVERBROAD-AGENCY`: agent permissions must match the proof lane.
- `HLT-013-RENDERED-UX-GAP`: web surfaces need artifact-backed rendered UX evidence, not only typechecks and happy-path browser tests.
- `HLT-014-A11Y-GAP`: changed UI surfaces need accessibility evidence.
- `HLT-015-CONTEXT-SETUP-GAP`: setup and context routing must be deterministic.
- `HLT-016-SUPPLY-CHAIN-DRIFT`: dependency and provenance changes need review evidence.
- `HLT-017-OPAQUE-OBSERVABILITY`: boundary failures need repairable telemetry.
- `HLT-018-PERF-CONCURRENCY-DRIFT`: performance and concurrency risk needs proof.
- `HLT-019-STREAMING-RUNTIME-DRIFT`: broker clients and Kafka stack identity must stay behind adapter boundaries or dated exceptions.
- `HLT-020-CI-HARDENING-GAP`: CI workflow permissions, action pinning, and proof posture gaps need repair.
- `HLT-021-DESTRUCTIVE-MIGRATION`: destructive SQL under migration paths needs documented rollback, backfill, staged deploy, lock, or explicit safety evidence.

## Artifact Map

| Artifact | Role | Version binding |
| --- | --- | --- |
| `paper/jankurai.tex` | TeX wrapper and canonical paper entrypoint | `paper_edition` |
| `paper/tex/` | TeX frontmatter, sections, appendices | `paper_edition` |
| `paper/jankurai.pdf` | Generated paper render | `paper_edition` |
| `paper/jankurai.md` | Agent-readable paper companion | `paper_edition` |
| `packages/ux-qa/` | Optional Playwright rendered-UX geometry runtime | `auditor_version` |
| `docs/agent-native-standard.md` | Full coding standard | `standard_version` |
| `agent/JANKURAI_STANDARD.md` | Short agent bootstrap | `standard_version` |
| `agent/standard-version.toml` | Canonical version manifest | all versions |
| `agent/repo-score.json` | Canonical audit report JSON | `standard_version`, `auditor_version`, `schema_version`, `paper_edition` |
| `agent/repo-score.md` | Human-readable audit and repair brief | `standard_version`, `auditor_version`, `schema_version`, `paper_edition` |
| `target/jankurai/evidence-index.json` | Proof evidence index from `jankurai prove` | proof schema |
| `target/jankurai/security/evidence.json` | Normalized security lane evidence from `jankurai security run` | security evidence schema |

## Adoption Levels

| Level | Purpose |
| --- | --- |
| `agents` | Install only `AGENTS.md`, `agent/JANKURAI_STANDARD.md`, `agent/MASTER_PLAN.md`, and provider adapters. |
| `score` | Add owner/test/generated-zone/proof/audit/version manifests and minimal local recipes. |
| `ci` | Add observe-mode workflow, security policy, and stub evidence path without a score gate. |
| `full` | Preserve the selected profile's full scaffold behavior. |
| `ratchet` | Explicit CI mode after a reviewed baseline; blocks regression. |

## Command Map

```bash
jankurai adopt . --profile auto --mode observe --out target/jankurai/adoption-plan.json --md target/jankurai/adoption-plan.md
jankurai init . --level agents --dry-run --plan-json target/jankurai/init-agents.json
jankurai init . --level agents --yes
jankurai init . --level score --yes
jankurai audit . --mode advisory --json target/jankurai/repo-score.json --md target/jankurai/repo-score.md
jankurai init . --level ci --yes
jankurai update . --check --out target/jankurai/update/update-plan.json --md target/jankurai/update/update-plan.md
jankurai doctor . --fail-on high --json target/jankurai/doctor.json --md target/jankurai/doctor.md
jankurai context-pack . --task "repair changed docs" --changed README.md --out target/jankurai/context-pack.json --md target/jankurai/context-pack.md
jankurai adapters verify .
jankurai adapters sync . --ide all --dry-run
jankurai agent verify .
jankurai hooks install . --dry-run
jankurai ci install . --github --mode observe --dry-run
jankurai ci install . --github --mode ratchet --baseline target/jankurai/baseline-score.json
```

```bash
jankurai issues export . --format jsonl --out target/jankurai/issues.jsonl
jankurai lane . --changed README.md --out target/jankurai/proof-plan.json --md target/jankurai/proof-plan.md
jankurai proof . --changed README.md --out target/jankurai/proof-plan.json --md target/jankurai/proof-plan.md
jankurai prove . --changed README.md --plan-out target/jankurai/proof-plan.json --plan-md target/jankurai/proof-plan.md
jankurai proof-verify . --plan target/jankurai/proof-plan.json --evidence-index target/jankurai/evidence-index.json --out target/jankurai/proof-verify.json --md target/jankurai/proof-verify.md
jankurai repair-plan . --from target/jankurai/repo-score.json --out target/jankurai/repair-plan.json --md target/jankurai/repair-plan.md
jankurai repair . --plan target/jankurai/repair-plan.json --dry-run --out target/jankurai/repair-run.json --md target/jankurai/repair-run.md
jankurai optimize . --mode token --out target/jankurai/optimize.json --md target/jankurai/optimize.md
jankurai exceptions expire . --strict --out target/jankurai/exceptions.json --md target/jankurai/exceptions.md
jankurai migrate . --analyze --out target/jankurai/migration-report.json --md target/jankurai/migration-report.md
jankurai cell . --cell-id background-job --mode prove --out target/jankurai/background-job.json --md target/jankurai/background-job.md
jankurai security run . --out target/jankurai/security/evidence.json
jankurai rust map . --out-dir target/jankurai/rust
jankurai rust witness build . --out target/jankurai/rust/witness-graph.json
jankurai rust diagnose . --out target/jankurai/rust/compile-packets.json
jankurai explain HLT-004-UNMAPPED-PROOF
jankurai bench . --out target/jankurai/p12-benchmark-report.json --md target/jankurai/p12-benchmark-report.md
jankurai certify . --out target/jankurai/p12-certification.json --md target/jankurai/p12-certification.md
jankurai govern . --out target/jankurai/p12-governance-policy.json --md target/jankurai/p12-governance-policy.md
jankurai publish . --certification target/jankurai/p12-certification.json --benchmark target/jankurai/p12-benchmark-report.json --governance target/jankurai/p12-governance-policy.json --out target/jankurai/public/p12-public-evidence.json --md target/jankurai/public/p12-public-evidence.md
```

## Build and Validation

```bash
just versions
just fast
just ux-qa
just paper
just score
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
