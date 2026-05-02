# Humans Were the Bug

Subtitle: **From Vibe Coding to Agent-Native Engineering**

Paper edition: `2026.05-ed3`

Standard version: `0.4.0`

Canonical source: `paper/humanlint.tex` plus `paper/tex/`

Rendered artifact: `paper/humanlint.pdf`

This Markdown file is an agent companion, not the canonical source. TeX remains the source of truth for the paper.

Naming policy: paper artifacts use the `humanlint.*` prefix. Do not create `main.md`, `main.tex`, or `main.pdf` anywhere in this repo.

## Executive Abstract

AI coding moves the bottleneck from typing plausible code to trustworthy merge. Agent-native engineering treats the repository as a verification interface for generated code: ownership, proof lanes, generated zones, security gates, repair receipts, and versioned artifacts must be machine-readable.

`humanlint` is the control plane for repositories that claim conformance. It defines conformance levels, stable rule IDs, audit output, CI modes, version bindings, and repair queues. The default stack recommendation remains Rust core, TypeScript/React/Vite product surface, PostgreSQL durable truth, generated contracts, and bounded Python for AI/data service work.

## Section Map

1. New Bottleneck: Trustworthy Merge
2. Definitions and Threat Model
3. Languages as Bottleneck Compression
4. Technical Promise Versus Standard Gravity
5. Vibe-Coding Fault Taxonomy
6. humanlint Standard and Conformance
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

## Core Claims

- The scarce act is trustworthy merge, not first-draft code generation.
- A repository claiming humanlint conformance must expose auditable ownership, proof routing, generated-zone policy, version metadata, and repair evidence.
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

## Artifact Map

| Artifact | Role | Version binding |
| --- | --- | --- |
| `paper/humanlint.tex` | TeX wrapper and canonical paper entrypoint | `paper_edition` |
| `paper/tex/` | TeX frontmatter, sections, appendices | `paper_edition` |
| `paper/humanlint.pdf` | Generated paper render | `paper_edition` |
| `paper/humanlint.md` | Agent-readable paper companion | `paper_edition` |
| `packages/ux-qa/` | Optional Playwright rendered-UX geometry runtime | `auditor_version` |
| `docs/agent-native-standard.md` | Full coding standard | `standard_version` |
| `agent/HUMANLINT_STANDARD.md` | Short agent bootstrap | `standard_version` |
| `agent/standard-version.toml` | Canonical version manifest | all versions |

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
cargo run -p humanlint -- . --json agent/repo-score.json --md agent/repo-score.md
```

The paper lane is:

```bash
latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=paper paper/humanlint.tex
```
