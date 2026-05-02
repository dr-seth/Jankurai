# Jankurai Moonshot Phase Index

Status: complete
Owner: standard
Last reviewed: 2026-05-02
Applies to: all files under `tips/phases/`

## Purpose

This file is the canonical router for executing the Jankurai moonshot phases. It is the first stop for a fresh session, a parallel MCP worker, or a maintainer deciding which phase can safely start next.

It does not implement any phase. It locks the execution order, dependency blockers, shared contracts, write scopes, validation lanes, handoff evidence, and cleanup policy for the phase plans in this directory.

For agent execution, the compact `MASTER_PLAN` contract is `agent/MASTER_PLAN.md` plus this index plus the active phase file. Agents asked to progress `MASTER_PLAN` must start with `agent/MASTER_PLAN.md`, then return here for the detailed phase roadmap.

## Phase 00 Goal

Phase 00 replaces the thinner phase index with a master execution plan. The goal is to make Phase 01 and every later phase start from the same routing facts:

- phase order is explicit and stable
- dependency blockers are visible before work starts
- write authority is scoped before files are touched
- shared contracts are named before parallel work splits
- validation commands are selected from changed paths
- handoff evidence is required at phase completion

Phase 00 is docs-only. It changes only `tips/phases/00-phase-index.md`.

## Read-First Ritual

Before implementing any phase, read these in order:

1. `agent/JANKURAI_STANDARD.md`
2. `agent/MASTER_PLAN.md`
3. `docs/agent-native-standard.md`
4. `docs/moonshot.md`
5. `tips/phases/00-phase-index.md`
6. the target phase file under `tips/phases/`
7. `agent/owner-map.json`
8. `agent/test-map.json`
9. `agent/generated-zones.toml`
10. `agent/standard-version.toml`

For this Phase 00 index-only edit, do not update `agent/owner-map.json` or `agent/test-map.json`. The existing `tips/` route already covers this file. Update those maps only when a later phase adds new durable paths.

## Current Baseline

As of 2026-05-02, the repo already has:

- Rust CLI package at `crates/jankurai/`
- commands for audit, init, doctor, CI install, issue export, explain, versions, adapter verification/sync, and UX passthrough
- JSON, Markdown, SARIF, JUnit-ish, GitHub summary, and repair queue exports
- machine-readable agent files under `agent/`
- UX QA runtime under `packages/ux-qa/`
- canonical paper and standard docs
- `docs/moonshot.md` as the north star for the phase sequence
- phase files `01` through `13` under `tips/phases/`

Treat the existing dirty worktree as project state. Do not revert unrelated changes. Do not update the paper until all phase plans are complete.

## Repository Constraints

- Do not edit `reference/`; it is read-only source material.
- Do not hand-edit generated artifacts. Change the generator or source and regenerate.
- Keep new durable docs under `docs/`, `agent/`, or `tips/phases/`.
- Keep root guidance short; put durable detail in `docs/` or `agent/`.
- Keep phase work inside the allowed write scope for that phase.
- Update `agent/owner-map.json` and `agent/test-map.json` when new durable paths are added.
- Update `agent/generated-zones.toml` when a phase creates a generated output.
- Add or update schemas when a phase creates machine-readable artifacts.
- Use `just fast` as the required validation for this docs-only index change.
- Use stricter validation for later phases when their touched paths require it.
- Never mark a phase complete without a phase completion receipt.
- Treat `tips/phases/logs/` as the canonical tracked append-only phase history.
- Keep `target/jankurai/` for volatile proof receipts, generated evidence, command logs, and artifacts only.

## Phase Dependency Graph

```text
00 phase index
  -> 01 standard stabilization
       -> 02 rule engine and semantic oracle
       -> 03 proof router and evidence ledger
       -> 04 init profiles and golden repos
       -> 05 UX proof platform
       -> 06 security supply chain and compliance evidence
       -> 07 contracts DB and generated boundaries
       -> 08 agent context and repair

02 -> 03, 07, 08, 11
03 -> 04, 05, 06, 07, 08, 11, 13
04 + 05 + 06 + 07 + 08 -> 09 reference product platform
04 + 07 + 09 -> 10 reuse registry certified cells
02 + 03 + 07 + 08 -> 11 migration engine
01 through 11 -> 12 benchmark certification and governance
03 + 08 + 10 + 11 + 12 -> 13 autonomous repair and optimization
```

## Phase Table

| Phase | Objective | Dependency blockers | Allowed write scope | Shared contracts | Parallelization status | Required validation | Handoff evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 01 Standard Stabilization | Make v0.4 foundations coherent, versioned, defensible, and release-ready before new features expand. | Phase 00 complete; no prior implementation phase. | `agent/`, `docs/`, `schemas/`, `crates/jankurai/`, root validation metadata, and generated score outputs by command only. No paper edits. | report schema, version manifest, owner/test maps, generated zones, release receipts. | Yes; docs/schema and CLI report work are already split across the implemented surfaces. | `just fast`; add `cargo test -p jankurai`, `just versions`, or schema checks when touched paths require them. | compatibility notes, updated routing maps for new paths, release evidence convention, validation output. |
| 02 Rule Engine And Semantic Oracle | Move audit from centralized text heuristics toward a versioned rule registry with semantic ownership and boundary checks. | Phase 01 report compatibility decisions. | `crates/jankurai/src/audit/`, rule metadata, `agent/boundaries.toml`, schemas, tests, and supporting docs. | stable rule IDs, rule registry schema, boundary oracle input format, report compatibility. | Yes after rule metadata contract is sketched. | `just fast`; add focused Rust tests for each analyzer and regression fixture. | rule registry contract, analyzer fixtures, finding examples, compatibility proof. |
| 03 Proof Router And Evidence Ledger | Make Jankurai choose the smallest sufficient proof lanes and record proof receipts that agents can reuse. | Phase 01 report stability; Phase 02 metadata is useful but not a hard start blocker. | proof/lane commands, `agent/test-map.json`, `agent/proof-lanes.toml`, evidence schemas, receipt storage docs, Rust tests. | proof plan schema, evidence receipt schema, owner/test maps, lane names. | Yes; planner and receipt surfaces are split enough for parallel proof workers. | `just fast`; add CLI tests for changed path routing and receipt parsing. | proof plan examples, receipt examples, changed-path routing evidence, skipped-lane rationale. |
| 04 Init Profiles And Golden Repos | Turn `jankurai init` into a profile-driven repo generator and golden repo creation layer. | Phase 01; benefits from Phase 03 proof routing. | init command, profile manifests, templates, generated repo fixtures, docs, schemas, tests. | profile manifest schema, generator contract, conflict/merge policy, generated-zone declarations. | Yes after generator contract is locked. | `just fast`; add init golden tests and generated repo audit checks. | profile contract, generated fixture proof, rerun/idempotency evidence, docs for profile selection. |
| 05 UX Proof Platform | Make rendered UX proof a first-class lane with deterministic browser, geometry, ARIA, accessibility, and state evidence. | Phase 01; benefits from Phase 03 receipts. | `packages/ux-qa/`, UX config, UX receipt schemas, Rust ingestion, docs, tests. | UX policy schema, receipt schema, viewport/story matrix, artifact path convention. | Yes; TypeScript runtime, Rust ingestion, and docs can split after receipt schema locks. | `just fast`; add `npm --workspace @jankurai/ux-qa run build` and `npm --workspace @jankurai/ux-qa run test`. | UX artifact examples, receipt output, route/story coverage docs, integration evidence. |
| 06 Security Supply Chain And Compliance Evidence | Replace security theater with parseable security, supply-chain, provenance, and compliance evidence. | Phase 01; benefits from Phase 02 metadata and Phase 03 ledger. | security commands, `.github/`, ops/security docs, evidence schemas, scanner config, tests. Never add secrets. | security receipt schema, scanner matrix, control map, CI evidence contract. | Yes; tool matrix, normalization, CI hardening, and compliance docs can split. | `just fast`; add available security lane checks when tool changes require them. | scanner matrix, normalized evidence examples, CI policy proof, compliance language review. |
| 07 Contracts DB And Generated Boundaries | Enforce public API, event, generated client, DB migration, and durable truth boundaries. | Phase 01; strongly benefits from Phase 02 and Phase 03. | contracts, schemas, DB policy docs, generated-zone metadata, analyzer code, tests. Do not hand-edit generated outputs. | contract source metadata, generated-zone contract, DB migration safety contract, event schema policy. | Yes after shared contract metadata is locked. | `just fast`; add contract, DB, and analyzer tests for touched surfaces. | drift examples, migration safety evidence, generated-zone reproduction proof, report integration. |
| 08 Agent Context And Repair | Make Jankurai a control plane for bounded agent context, permissions, repair packets, and adapters. | Phase 01; benefits from Phase 02 and Phase 03. | `agent/`, adapter files, context-pack and repair-plan commands, permission profiles, MCP design docs, tests. | context pack schema, repair packet schema, permission profile contract, source hierarchy. | Yes; adapters, context/repair schemas, policy checks, and docs can split after schema locks. | `just fast`; add adapter verification and CLI tests where changed. | token-minimized context example, repair packet example, permission profile proof, adapter verification. |
| 09 Reference Product Platform | Build the canonical Jankurai-native fullstack SaaS reference proving the COLD stack end to end. | Requires phases 04, 05, 06, 07, and 08. | declared golden repo path, apps/API/domain/adapters/contracts/db/UX/security/observability docs and fixtures. | golden repo contract, generated client contract, DB truth contract, UX/security/proof receipts. | Yes; the in-tree example scaffold and fixture routing now split cleanly by surface. | `just fast` for Jankurai repo plus generated repo proof lane defined by the phase. | golden repo score, end-to-end proof receipts, minimal exceptions, stack doctrine evidence. |
| 10 Reuse Registry Certified Cells | Provide certified reusable product and engineering cells so agents stop rebuilding common primitives badly. | Requires phases 04 and 07; strongly benefits from Phase 09 and Phase 08. | registry manifests, cell templates, contracts, migrations, UI, tests, docs, certification harness. | cell manifest schema, install contract, proof receipt, upgrade path contract. | Yes after registry manifest and install contract lock. | `just fast`; add cell install, generation, and certification tests. | first certified cells, install proof, reproducible certification receipt, upgrade notes. |
| 11 Migration Engine | Make legacy modernization measurable, sliced, and agent-executable without reckless rewrites. | Requires phases 02, 03, 07, and 08; benefits from Phase 10 cells. | migrate commands, analyzers, liability scoring, plan schemas, fixtures, docs, tests. | legacy inventory schema, liability score model, slice plan schema, equivalence proof contract. | Yes; inventory, scoring, planning, and docs can split after schemas lock. | `just fast`; add migration fixture tests and scoring snapshot/compatibility checks. | sample liability score, migration plan, equivalence proof template, rollback guidance. |
| 12 Benchmark Certification And Governance | Prove the thesis publicly with benchmarks, conformance badges, signed attestations, and governance. | Requires real surfaces from phases 01 through 11. | benchmark corpus, certification schemas, badge/report outputs, governance docs, release evidence. | benchmark manifest, attestation schema, badge fingerprint, governance policy. | Yes; benchmark corpus, metrics, governance, badges, and org reporting can split. | `just fast`; add benchmark/certification tests and report compatibility checks. | benchmark results, signed or reproducible attestations, badge evidence, governance process. |
| 13 Autonomous Repair And Optimization | Add constrained auto-repair, auto-PRs, exception expiry, and optimization after proof systems mature. | Requires phases 03, 08, 10, 11, and 12. | repair eligibility, dry-run planner, bounded patch execution, PR workflow, optimization commands, exception expiry logic. | repair eligibility policy, dry-run patch contract, PR evidence receipt, exception expiry schema. | Yes; repair planning is dry-run only, with write paths still gated by the proof and permission model. | `just fast`; add dry-run, sandbox, permission, and regression tests before enabling writes. | dry-run repair examples, bounded patch proof, opt-in policy, exception expiry report. |

## Phase Readiness Gates

Each phase may start only when these gates are true:

| Phase | Start gate |
| --- | --- |
| 01 | Phase 00 is complete, `docs/moonshot.md` exists, and report compatibility is the first decision. |
| 02 | Phase 01 has locked version/report compatibility and stable rule ID expectations. |
| 03 | Phase 01 has stable report contracts; Phase 02 metadata is available or the phase documents the temporary mapping source. |
| 04 | Phase 01 is complete; profile manifest and generator contracts are drafted before template work splits. |
| 05 | Phase 01 is complete; UX receipt schema is drafted before runtime and Rust ingestion split. |
| 06 | Phase 01 is complete; security evidence language avoids claiming compliance that is not proven. |
| 07 | Phase 01 is complete; generated-zone and contract metadata rules are stable enough to enforce. |
| 08 | Phase 01 is complete; source hierarchy and permission profile terms are explicit. |
| 09 | Phases 04, 05, 06, 07, and 08 have partial implementation slices and receipts. |
| 10 | Phases 04 and 07 have partial implementation slices; Phase 09 exists as the proving ground or the phase records why it can proceed without it. |
| 11 | Phases 02, 03, 07, and 08 have the partial rule, proof, contract, and repair surfaces needed to start migration planning. |
| 12 | Phases 01 through 11 have enough partial surfaces for benchmarks and certification to measure. |
| 13 | Phases 03, 08, 10, 11, and 12 have partial repair, registry, migration, and governance surfaces. |

All phases must also satisfy these universal gates:

- owned paths are declared before edits
- forbidden paths are declared before edits
- new durable paths are added to owner/test maps
- generated outputs are declared in `agent/generated-zones.toml`
- public interface or schema changes include compatibility notes
- validation is chosen from `agent/test-map.json` and the phase plan

## Parallel MCP Work Model

Use parallel MCP workers only when write scopes are disjoint and the merge path is already clear. A parallel worker must be given a packet with owned paths, forbidden paths, input contracts, output contracts, validation commands, expected artifacts, stop conditions, merge order, and residual risk.

Safe parallel patterns:

- one worker owns Rust CLI internals while another owns docs or schemas
- one worker owns UX QA package changes while another owns Rust ingestion after the UX receipt schema locks
- one worker owns security evidence normalization while another owns compliance docs
- one worker owns benchmark fixtures while another owns report rendering
- one worker owns generated template source while another owns init command logic after the generator contract locks

Unsafe parallel patterns:

- multiple workers editing the same rule registry schema without a locked contract
- multiple workers changing report JSON shape without a compatibility plan
- multiple workers changing `agent/test-map.json` independently
- multiple workers editing generated outputs directly
- multiple workers changing paper content while phase plans are still incomplete

Merge order should follow contracts before implementations, implementations before generated outputs, generated outputs before docs that cite them, and docs before final validation receipts.

## Parallel Work Packet Template

- Agent name:
- Phase:
- Scope:
- Owned paths:
- Forbidden paths:
- Input contracts:
- Output contracts:
- Log path:
- Required commands:
- Expected artifacts:
- Stop conditions:
- Merge order:
- Handoff expectations:
- Residual risk:

## Phase Log Routing

Tracked phase logs live under `tips/phases/logs/`. Use one file per phase and append start, progress, and finish entries for every attempt. New entries use:

```text
timestamp_utc | actor/tool | phase | action | changed_paths | validation | artifacts | git_sha | residual_risk
```

The phase log is the durable cross-agent history. Proof receipts, generated reports, screenshots, and command output remain volatile evidence under `target/jankurai/` and should be cited from the log when relevant.

## Validation Matrix

| Change surface | Minimum validation | Add when relevant |
| --- | --- | --- |
| Phase 00 index only | `just fast` | None unless this file grows past standard limits. |
| `tips/phases/` phase plan docs | `just fast` | `just score` when routing maps or generated zones change. |
| `agent/` maps and policy | `just fast` | `just score`, adapter verification, or schema checks for machine-readable changes. |
| Rust CLI behavior | `cargo test -p jankurai` and `just fast` | focused integration tests, `just versions`, report compatibility fixtures. |
| Schemas or contracts | `just fast` | schema validation, contract compatibility checks, generated client regeneration. |
| Generated artifacts | generator command plus `just fast` | generated-zone proof and diff evidence. |
| UX QA runtime | `npm --workspace @jankurai/ux-qa run build` and `npm --workspace @jankurai/ux-qa run test` | browser artifact review for changed rendered surfaces. |
| Security and supply chain | `just fast` | available security lane commands from `just security` or the phase-specific scanner matrix. |
| Paper | `just paper` | only after all phase plans are complete and paper work is explicitly in scope. |
| Release or certification | phase-specific full proof lane | signed or reproducible receipt bundle. |

## Cleanup And Deprecation Policy

- Do not update the paper until all phase plans are complete.
- No cleanup or deprecation is required for Phase 00 beyond replacing the thinner current index content.
- Do not rename phase files after implementation work starts unless the rename is a planned migration recorded here.
- If a later phase deprecates a command, schema, rule ID, or artifact path, it must provide compatibility notes and a migration path.
- Generated artifacts are never cleaned up by hand. Remove or change their sources and run the recorded generator.
- Remove obsolete docs only when their replacement is linked, routed, and covered by validation.
- Existing dirty worktree changes are project state, not clutter to clean during unrelated work.

## Phase Completion Receipt

- Phase completed: 00 phase index
- Files changed: `tips/phases/00-phase-index.md`
- Public interfaces changed: none
- Schemas changed: none
- Generated artifacts: none
- Validation commands: `just fast`
- Results: passed
- Skipped validation: none
- Exceptions created: none
- Follow-up phases unblocked: 01 standard stabilization, plus the documented downstream phase order

## Ready For Phase 01 Checklist

- `docs/moonshot.md` exists and is the north star.
- `tips/phases/01-standard-stabilization.md` exists.
- Phase order is locked.
- Validation command for docs-only phase index work is `just fast`.
- No paper work is required.
- No cleanup/deprecation is required beyond replacing the thinner current index content.
