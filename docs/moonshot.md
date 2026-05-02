# Humanlint Cold Standard: The End Of Ungoverned Software

Status: planning source
Owner: standard
Last reviewed: 2026-05-02
Applies to: humanlint roadmap, standard, audit CLI, UX QA package, future templates, future registry

## Executive Summary

Humanlint is not a style linter. The moonshot is to make Humanlint the default operating standard for agent-native software: a versioned, machine-readable control plane that makes unprovable code unmergeable.

The project exists to end vibe coding by making every serious repository constrained, observable, localized, and deterministic. Every file has an owner. Every boundary has a contract. Every generated artifact has a declared source and regeneration command. Every changed path maps to the smallest sufficient proof lane. Every exception has an owner, expiry, and migration path. Every agent action leaves evidence. Every repeated primitive should become reusable instead of rebuilt badly.

The human role moves up the stack. Humans define product intent, taste, ethics, risk tolerance, constraints, customer truth, and ambition. Humanlint enforces how the repo is built, proven, secured, rendered, observed, migrated, and kept from drifting into entropy.

The simplest product promise:

```text
Humans state intent.
Humanlint selects the safe standard.
Agents build inside bounded authority.
Proof lanes validate the change.
Audit evidence records the result.
Drift is rejected.
Exceptions expire.
Reusable primitives replace repeated bad rebuilds.
```

## The Thing To Kill

Humanlint does not exist to attack learning, craft, curiosity, or healthy engineering judgment. It exists to kill unproven entropy.

The target is the waste pattern:

- ambiguous ownership
- bad stack selection by habit
- duplicated business truth
- handwritten API clients and DTO mirrors
- scattered SQL and wrong-layer DB access
- business logic in UI components
- Python product truth outside bounded AI/data zones
- generated files edited by hand
- silent fallbacks and catch-all exceptions
- TODO-driven production behavior
- weak tests that create false green
- missing rendered UX evidence
- missing security and supply-chain receipts
- unbounded agent permissions
- token-heavy instruction sprawl
- legacy exceptions without expiry
- primitives rebuilt from scratch for no product advantage

The hard line:

```text
No proof, no merge.
No owner, no merge.
No contract, no boundary.
No generated-zone declaration, no generated artifact.
No security evidence, no deploy.
No UI proof, no critical UI release.
No exception without expiry.
```

## COLD Standard

COLD means **Constrained, Observable, Localized, Deterministic**.

Constrained:
- Agents do not spray code across the repo.
- File ownership, dependency direction, generated zones, proof lanes, security expectations, and exception rules define safe edit authority.
- Alternate stacks and nonstandard patterns are allowed only through explicit exceptions with evidence and migration plans.

Observable:
- Every important runtime boundary emits structured traces, metrics, logs, correlation IDs, typed errors, and repair hints.
- Every proof lane emits machine-readable receipts.
- Security, UX, contract, DB, performance, and release evidence are durable artifacts, not tribal memory.

Localized:
- Each behavior has one owner cell.
- Changes map to narrow files, tests, contracts, and owners.
- Duplicate truth is treated as a correctness risk because agents will patch one copy and miss another.

Deterministic:
- The repo produces JSON, Markdown, SARIF, JUnit, screenshots, ARIA snapshots, SBOMs, provenance, benchmark deltas, and repair queues.
- Passing the wrong tests is not success.
- Visual and UX claims require rendered artifacts and deterministic checks where possible.

## Default Stack Doctrine

Humanlint is deliberately opinionated. Agents operate better in low-entropy environments, and business outcomes improve when stack decisions are made by proof economics instead of yesterday's habits.

Default stack:

| Layer | Default | Owns | Must not own |
| --- | --- | --- | --- |
| Core/domain | Rust | IDs, invariants, state machines, typed errors, pure decisions | I/O, environment, DB, framework types, UI |
| API edge | Rust Axum/Tower or equivalent Rust edge | request extraction, response mapping, auth/session bridge, tracing boundary | domain rules, scattered SQL |
| Application | Rust | commands, queries, authz, idempotency, transaction orchestration | UI, provider details |
| Adapters | Rust | PostgreSQL, queues, external APIs, filesystem, env, secrets | business rules |
| Product surface | TypeScript, React, Vite | UI, forms, local validation, generated clients, browser proof | secrets, durable truth, core authz, direct DB writes |
| Durable truth | PostgreSQL | constraints, migrations, indexes, transactions, RLS where useful | hidden app-only invariants |
| Contracts | OpenAPI, JSON Schema, Protobuf where appropriate | boundary truth, generated clients, drift checks | handwritten mirrors |
| AI/data | bounded Python | model wrappers, embeddings, evals, data transforms, notebooks with policy | product truth, authz, production DB ownership |
| UX proof | Playwright, Storybook, Humanlint UX QA | screenshots, ARIA, geometry, accessibility, visual baseline evidence | subjective-only approval for deterministic failures |
| Observability | OpenTelemetry and structured errors | traces, metrics, logs, correlation IDs, repairable runtime evidence | opaque failures |
| Security | real scan and provenance lanes | secrets, SAST/SCA, dependency policy, SBOM, CI hardening, release evidence | optional security theater |

This doctrine is not "never use anything else." It is:

```text
The approved stack is the default.
Deviations must beat the rubric or carry a visible exception cost.
```

## Perfect Code, Defined Practically

Humanlint cannot promise mystical code that never fails under unspecified requirements. It can define a useful engineering target.

Perfect code is code that is optimal against its declared:

- product specification
- architecture boundary
- generated contract
- test and proof obligation
- threat model
- performance and memory budget
- UX contract
- accessibility requirement
- observability contract
- data truth policy
- exception policy
- change policy

Perfect code has explicit claims, machine-readable ownership, generated boundary truth, enough tests to prove the changed behavior, runtime evidence for repair, and bounded future change paths. It is hostile to drift.

## Product Surface

The CLI remains the source of truth. Humanlint now has a mixed surface: some commands are implemented, some are planner-only or dry-run-only, and some are still roadmap surfaces.

| Command | State | Purpose |
| --- | --- | --- |
| `humanlint init` | implemented | create the repo constitution, stack profile, agent files, lanes, CI, docs, and templates |
| `humanlint doctor` | implemented | verify local environment, required artifacts, stale score, path leaks, receipt exports, proof ledger, security/context-pack/repair-plan artifacts when present, **`agent/boundaries.toml`** and **`agent/ux-qa.toml`** (if present) against their JSON schemas, and lane health |
| `humanlint audit` | implemented | score the repo, detect hard caps, emit JSON/Markdown/SARIF/JUnit, and build the repair queue |
| `humanlint ci install` | implemented | install CI workflows and PR surfaces |
| `humanlint ux` | implemented | run rendered UX, geometry, screenshots, ARIA, accessibility, Storybook, and visual evidence workflows |
| `humanlint adapters verify` / `sync` | implemented | verify or refresh thin adapter pointers |
| `humanlint agent verify` | implemented | verify adapter drift, canonical pointers, and agent-facing policy surfaces |
| `humanlint context-pack` | implemented | produce a token-minimized task packet from owner map, test map, proof lanes, and heuristics; JSON validated on write against `schemas/context-pack.schema.json` |
| `humanlint repair-plan` | implemented | convert audit/report findings into repair packets; JSON validated on write against `schemas/repair-plan.schema.json` (v1 packet-centric envelope; planner-only fields such as `plan_mode` deferred) |
| `humanlint repair --dry-run` | dry-run only | review a bounded repair plan without mutating files |
| `humanlint registry` | planner-only | plan reusable product and engineering cells |
| `humanlint cell` | planner-only | plan a reusable cell install or certification slice |
| `humanlint migrate` | planner-only | score and slice legacy systems toward the default stack |
| `humanlint bench` | planner-only | plan benchmark and corpus outputs |
| `humanlint certify` | planner-only | plan conformance and release evidence |
| `humanlint govern` | planner-only | plan ratchets, exceptions, and policy updates |
| `humanlint lane` / `humanlint proof` | implemented | map changed files (`--changed` / `--changed-from`) to test-map and proof-lanes commands; emit proof plan JSON/Markdown validated against `schemas/proof-plan.schema.json` |
| `humanlint prove` | implemented | execute a validated proof plan; write receipts (`schemas/proof-receipt.schema.json`), logs under `target/humanlint/logs/`, and evidence index (`schemas/evidence-index.schema.json`); commands must match `agent/proof-lanes.toml` and `agent/test-map.json` unless `--allow-unsigned-commands` is used with `HUMANLINT_ALLOW_UNSIGNED_PROOF_COMMANDS=1` |
| `humanlint security` | partial | `humanlint security run`: execute the canonical bash security lane (`tools/security-lane.sh` by default), write a combined log and schema-valid evidence JSON (`schemas/security-evidence.schema.json`, default `target/humanlint/security/evidence.json`); use `--strict` to set `HUMANLINT_SECURITY_STRICT=1` for the child process. Full scanner matrix, policy file, and audit ingestion remain planned. |
| `humanlint contracts` | planned | verify source contracts, generated clients, drift, compatibility, and error schemas |
| `humanlint db` | planned | verify migrations, constraints, rollback policy, query safety, and durable truth |

Every future command must produce a proof artifact or a decision artifact that the audit can consume.

## Reference Platform Contract

The reference product platform should stay small enough to inspect, prove, and regenerate. Preferred first location inside this repo:

```text
examples/perfect-web-api-db/
```

Rules:

- source truth lives in contracts, migrations, Rust backend code, React/Vite frontend code, and proof docs
- generated client, screenshots, test receipts, and other derived outputs stay in declared generated zones
- `reference/` stays read-only
- if the in-tree example grows past the repo context budget, move the golden repo out-of-tree and keep the contract and proof lane names stable

The goal is a canonical proving ground, not a demo toy.

## Proof Model

Humanlint should make proof cheaper, not heavier. The system should not blindly run everything on every change. It should route each change to the smallest lane set that covers risk.

Canonical lanes:

| Lane | Purpose |
| --- | --- |
| `setup` | toolchain, lockfile, generated artifact, and local service readiness |
| `fast` | deterministic local proof under two minutes where possible |
| `audit` | Humanlint score, hard caps, and repair queue |
| `contract` | OpenAPI, Protobuf, JSON Schema, generated clients, compatibility |
| `domain` | Rust domain unit, table, and property tests |
| `application` | commands, authz, idempotency, transaction, workflow tests |
| `db` | migrations, constraints, rollback, query checks, RLS where applicable |
| `api` | integration tests, transport boundary, auth/session behavior |
| `web` | TypeScript, React, component tests, Storybook states |
| `ux` | Playwright, geometry, screenshots, ARIA, accessibility, visual diffs |
| `security` | secrets, SAST/SCA, SBOM, provenance, workflow and dependency policy |
| `perf` | benchmarks, memory, concurrency, query plans, bundle budgets |
| `observability` | traces, logs, metrics, problem details, repair signals |
| `release` | signed full evidence pack for deployable artifacts |
| `full` | broad integration and E2E for high-risk or release changes |

Every proof receipt should include:

- lane
- command
- exit code
- elapsed time
- changed paths
- owner
- artifacts
- rule IDs covered
- skipped checks with reason
- residual risk

## Rendered UX Doctrine

The UX claim is intentionally sharp: a large class of repetitive UI QA should become deterministic proof infrastructure.

Human review still matters for taste, brand, copy nuance, and ambiguous product judgment. Humans should stop doing repetitive screenshot archaeology for issues machines can catch.

Critical UI changes should produce:

- route or story ID
- viewport matrix
- browser name
- screenshot
- crop for each deterministic violation
- ARIA snapshot
- accessibility scan output
- geometry report
- visual baseline decision
- focus and keyboard evidence
- loading, empty, error, permission-denied, and success state coverage
- artifact-backed merge decision

Deterministic geometry and accessibility failures block. Pixel baseline changes route to owner approval. AI or VLM visual opinions are advisory unless backed by deterministic evidence.

## Security And Compliance Doctrine

Humanlint must avoid fake compliance claims.

Correct claim:

```text
Humanlint can create SOC-ready engineering evidence by default.
Humanlint does not make an organization SOC 2 certified by itself.
```

Default evidence should include:

- change-management receipts
- access-control policy and audit log evidence
- vulnerability-management receipts
- secret scan receipts
- dependency inventory and scan receipts
- SBOMs
- signed provenance where applicable
- CI workflow hardening
- incident response runbooks
- backup and restore proof
- PII/data classification
- retention and deletion policy
- exception register
- release approvals

Security is not a dashboard. It is a lane with parseable artifacts, owners, thresholds, and repair paths.

## No-Sprawl Law

Humanlint must not become the sprawl it exists to defeat.

A tool is allowed only if it satisfies all conditions:

- catches a real defect class
- has a deterministic decision mode or clearly marked advisory mode
- can run locally and in CI
- emits parseable evidence
- maps to a Humanlint rule ID or score dimension
- has an owner
- has a proof lane
- has a repair path
- has a removal or replacement policy

No tool exists because it is fashionable. No output is orphaned. No manual dashboard inspection is required for merge-critical decisions.

## Reuse Registry Doctrine

The project should eventually ship certified reusable cells for common product and engineering primitives. This is where Humanlint can remove massive repeated waste.

Initial product cells:

- authentication
- sessions
- OAuth/OIDC
- RBAC
- organizations and teams
- tenant isolation
- admin console
- CRUD table and form
- audit log
- billing and subscriptions
- invoices
- feature flags
- webhooks
- file upload
- notifications and email
- search
- API keys
- rate limits
- background jobs
- idempotency
- data export and deletion
- settings pages

Initial engineering cells:

- generated contract pipeline
- DB migration safety
- OpenTelemetry setup
- typed error catalog
- security lane
- SBOM and provenance
- release workflow
- Storybook/Playwright/UX QA
- compliance evidence shell
- agent adapters
- context packs
- repair receipts

Every cell must include contracts, source, migrations where applicable, tests, UX proof where applicable, security assumptions, observability, docs, benchmarks when relevant, and an upgrade path.

## Conformance Direction

The current HL0-HL5 ladder remains the active standard. The moonshot direction extends it over time:

| Level | Meaning |
| --- | --- |
| HL0 | uncontrolled or unrouted |
| HL1 | advisory audit |
| HL2 | guarded critical caps |
| HL3 | score floor and high/critical blocking |
| HL4 | ratchet against regression |
| HL5 | release contract across audit, tests, security, contracts, DB, e2e, and versions |
| Future HL6 | contract-complete and proof-routed |
| Future HL7 | UX-proofed |
| Future HL8 | runtime-verifiable |
| Future HL9 | organization-native |
| Future HL10 | Cold Standard: machine-governed, reusable, self-improving software infrastructure |

Future levels must be versioned policy, not marketing labels.

## CEO Interface

Humanlint should make executive questions concrete:

```text
What are we building?
Who uses it?
What data is sensitive?
What risk class are we in?
What latency matters?
What must never fail?
Which integrations are needed?
Which product primitives are differentiating?
Which primitives should be reused?
```

Humanlint should answer:

```text
Recommended stack
Architecture profile
Repo skeleton
Agent authority
Proof lanes
Security posture
Rendered UX evidence
Compliance evidence shell
Reusable cells
Migration risks
Release gates
Current exceptions
Cost of nonstandard choices
```

The CEO should not choose between random frameworks. The CEO should decide product intent and risk. Humanlint should make the safe engineering default the fastest path.

## Research And Benchmark Claim

The project must prove the moonshot empirically.

Humanlint benchmarks should measure:

- agent token usage per accepted task
- wrong-file edit rate
- time to trustworthy merge
- test selection accuracy
- false-green rate
- security issue escape rate
- UX issue escape rate
- patch size
- human review burden
- build and CI time
- duplicate primitive rebuild count
- migration slice completion time
- runtime cost and memory deltas

The public claim should be:

```text
Humanlint-native repos produce safer agent changes with fewer tokens, fewer wrong edits, fewer regressions, and better proof artifacts.
```

## Non-Goals

Humanlint should not:

- become a generic style linter
- install every tool in every repo
- claim compliance certification without organizational evidence
- ban all alternate stacks by ideology
- replace human product taste
- trust AI visual opinions as deterministic proof
- encourage broad auto-rewrites without bounded authority
- create contradictory agent instruction files
- make root instructions long
- hand-edit generated artifacts

## Final Vision

Humanlint is the Cold Standard for agent-native engineering: the default initialization system, audit engine, proof router, repair planner, security harness, UX QA system, contract enforcer, performance budgeter, compliance evidence generator, migration engine, and reuse registry for modern software.

It exists to end the era where every team reinvents the same fragile stack, repeats the same security mistakes, writes the same weak tests, hand-rolls the same contracts, ships the same layout bugs, forgets the same observability, and teaches every new agent a different messy repo.

The final ambition:

```text
Every serious codebase starts with Humanlint.
Every agent reads Humanlint.
Every merge proves itself through Humanlint.
Every exception expires through Humanlint.
Every reusable primitive improves through Humanlint.
Every substandard pattern becomes migrated, contained, or rejected.
```
