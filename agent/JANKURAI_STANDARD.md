# jankurai Standard Agent Bootstrap

Standard version: `0.7.0`
Published: `2026-05-04`
Full standard: `docs/agent-native-standard.md`
Version manifest: `agent/standard-version.toml`
Paper: `Jankurai: A Versioned Repository Conformance Standard for Trustworthy AI-Assisted Merge`
Public thesis line: `No proof, no merge; no receipt, no trust.`

## Prime Directive

Optimize for agent-verifiable engineering. Reject wrong code fast, localize failure, repair narrow scope, and leave evidence.

Target stack:

- Rust core
- TypeScript/React/Vite product surface
- PostgreSQL durable truth
- generated contracts
- exception-only Python AI/data service, only when an advanced ML/data library
  has no practical Rust, TypeScript, or service alternative

Implementation default: use Rust for repository tools, core logic, proof lanes,
and automation whenever practical. Agents must not create or expand Python for
repo tooling, proof lanes, product services, general backend glue,
authorization, or PostgreSQL access. The only allowed Python exception is rare:
advanced ML/data work that depends on a Python-only library, is boxed under
`python/ai-service`, and has a dated exception with owner, expiry, proof lane,
and a migration/containment plan.

## Start Ritual

Before edits:

- read this file
- read `docs/agent-native-standard.md` when policy detail matters
- inspect `agent/owner-map.json`
- inspect `agent/test-map.json`
- inspect `agent/generated-zones.toml`
- inspect `agent/standard-version.toml` for versioned artifacts
- inspect `agent/tool-adoption.toml` when rolling out new Jankurai-backed tool lanes
- check target file length before adding behavior
- search for existing owner and duplicate behavior

Do not edit outside the requested ownership scope.

## Conformance

Only repositories claiming jankurai conformance are bound by these levels:

| Level | Gate |
| --- | --- |
| `HL0` | unscored or unrouted |
| `HL1` | advisory audit |
| `HL2` | guarded critical caps |
| `HL3` | standard score floor plus high/critical blocking |
| `HL4` | ratchet against regression |
| `HL5` | release contract across audit, tests, security, contracts, DB, e2e, and versions |

## Hard Blocks

Stop or fix first when any condition is true:

- no root `AGENTS.md` or equivalent agent instructions
- no one-command fast validation
- path has no owner-map entry
- path has no test-map entry
- non-generated file exceeds hard LOC max without an exception
- generated file would need hand edit
- public API/schema changes without contract regeneration
- UI, exception-only Python, or domain code writes product truth directly
- Python is added without an approved advanced-ML/data exception
- Python owns product authorization, product truth, proof lanes, repo tooling,
  general backend glue, or production DB writes
- new silent fallback, broad catch, disabled test, or duplicate behavior
- product/runtime code contains future-hostile markers without allowlisted docs/generated/vendor/product-copy context or dated exception
- paper sources must not mention `tips/*.txt` file names; refer to corpus, row-family, or source-group labels instead
- high-risk change lacks security lane
- generated code changes auth/input/crypto/filesystem behavior without security proof
- secret-like values, prompt transcripts, MCP config, fixtures, or logs expose credentials or customer data
- trusted agent/tool policy contains prompt-injection, bypass, or overbroad permission language
- destructive migration lacks rollback, backfill, lock, and DB proof evidence (`HLT-021-DESTRUCTIVE-MIGRATION` when destructive SQL is present without documented safety markers documented in `docs/testing.md`)
- tests are skipped/focused/tautological/snapshot-only for changed behavior
- agent tool permissions are broader than the requested lane
- user-facing UI changes lack artifact-backed rendered UX proof on critical surfaces

## Stable Rule IDs

| Rule | Meaning |
| --- | --- |
| `HLT-001-DEAD-MARKER` | future-hostile product/runtime marker |
| `HLT-002-GENERATED-MUTATION` | generated output changed outside source regeneration |
| `HLT-003-OWNERLESS-PATH` | path has no owner-map route |
| `HLT-004-UNMAPPED-PROOF` | path has no test-map proof lane |
| `HLT-005-PYTHON-PRODUCT-TRUTH` | Python owns durable product behavior |
| `HLT-006-DIRECT-DB-WRONG-LAYER` | DB access appears outside adapters/db |
| `HLT-007-HANDWRITTEN-CONTRACT` | public API/client contract is mirrored by hand |
| `HLT-008-FALSE-GREEN-RISK` | passing lane does not prove changed behavior |
| `HLT-009-GENERATED-SECURITY` | generated security-sensitive code lacks security proof |
| `HLT-010-SECRET-SPRAWL` | secret-like value, env dump, fixture, or transcript leak |
| `HLT-011-PROMPT-INJECTION` | untrusted context changes trusted policy/tool behavior |
| `HLT-012-OVERBROAD-AGENCY` | agent/tool permissions exceed lane scope |
| `HLT-013-RENDERED-UX-GAP` | user-facing UI lacks rendered proof |
| `HLT-014-A11Y-GAP` | UI lacks accessibility proof for changed surface |
| `HLT-015-CONTEXT-SETUP-GAP` | setup/context routing is not deterministic |
| `HLT-016-SUPPLY-CHAIN-DRIFT` | dependency/provenance change lacks review evidence |
| `HLT-017-OPAQUE-OBSERVABILITY` | boundary failure lacks repairable telemetry |
| `HLT-018-PERF-CONCURRENCY-DRIFT` | performance/concurrency risk lacks proof |
| `HLT-019-STREAMING-RUNTIME-DRIFT` | broker client or Kafka stack identity escapes adapter boundaries |
| `HLT-020-CI-HARDENING-GAP` | CI workflow permissions, unpinned actions, or proof posture gaps |
| `HLT-021-DESTRUCTIVE-MIGRATION` | destructive SQL under migration paths without documented safety evidence |
| `HLT-022-AUTHZ-ISOLATION-GAP` | authorization or tenant/data isolation lacks negative proof |
| `HLT-023-INPUT-BOUNDARY-GAP` | unsafe input boundary or sink lacks deterministic negative proof |
| `HLT-024-AGENT-TOOL-SUPPLY-GAP` | agent tool, MCP, hook, or rule supply chain lacks trust evidence |
| `HLT-025-RELEASE-READINESS-GAP` | release or launch gate lacks artifact-backed readiness evidence |
| `HLT-026-COST-BUDGET-GAP` | unbounded paid work lacks budget, quota, or stop-condition evidence |
| `HLT-027-HUMAN-REVIEW-EVIDENCE-GAP` | review or proof claim lacks reproducible receipts |
| `HLT-028-BOUNDARY-EVIDENCE-GAP` | audited runtime boundary reclassification lacks deterministic evidence |
| `HLT-029-RUST-BAD-BEHAVIOR` | Rust code uses unsafe, unchecked, or dishonest APIs without local proof |
| `HLT-030-SQL-BAD-BEHAVIOR` | SQL code or migrations use unsafe string assembly or unchecked execution without proof |
| `HLT-031-TYPESCRIPT-BAD-BEHAVIOR` | TypeScript code uses unchecked boundary or runtime shortcuts without proof |
| `HLT-032-DOCKER-BAD-BEHAVIOR` | Docker or container build behavior hides unsafe or unreviewed execution steps |
| `HLT-033-PYTHON-BAD-BEHAVIOR` | Python code owns runtime behavior or unchecked data paths without an approved exception |
| `HLT-034-CI-BAD-BEHAVIOR` | CI workflows hide unsafe, unpinned, or nonblocking security and proof behavior |
| `HLT-035-GIT-BAD-BEHAVIOR` | Git automation or hooks use destructive, hidden-state, or unreviewed mutation behavior |

`HLT-029-RUST-BAD-BEHAVIOR` is detector-backed now. `HLT-030` through `HLT-035` are detector-backed catalog IDs in the language bad-behavior family.

## Ownership Boundaries

| Layer | Owns | Never owns |
| --- | --- | --- |
| `apps/web` | UI, forms, generated clients, browser tests | secrets, durable truth, core authz, direct DB |
| `apps/api` | HTTP/RPC edge, extraction, response mapping | domain rules, raw SQL decisions |
| `crates/domain` | IDs, invariants, pure decisions | IO, env, time, random, DB, framework types |
| `crates/application` | commands, authz, idempotency, transactions | UI, external protocol details |
| `crates/adapters` | DB, queue/streaming clients, external APIs, filesystem, env | domain rules, event schema ownership |
| `crates/workers` | jobs, backpressure, workflow glue | product truth outside application |
| `contracts` | OpenAPI/protobuf/JSON Schema and generated clients | handwritten drift |
| `db` | migrations, constraints, indexes, RLS | app-only durable invariants |
| `python/ai-service` | exception-only advanced ML/data library work behind typed boundaries | product truth, authz, production DB writes, proof lanes, repo tools, general backend glue |
| `ops` | CI, OTel, SBOM, SCA, secrets, provenance | hidden manual gates |

## Generated Zones

Never hand-edit generated files. Change source contract and regenerate.

Generated files must declare:

```text
Generated by: <tool> <version>
Source: <contract path>
Command: <regen command>
DO NOT EDIT BY HAND.
```

Structured generated artifacts that cannot legally carry comments, including
`agent/repo-score.json` and native lockfiles such as `package-lock.json`, must
carry equivalent machine-readable identity in their native schema. Required
identity includes generator/schema metadata and version fields for reports, or
the package-manager lockfile shape for lockfiles.

## Proof Lanes

Use `agent/test-map.json` to select the smallest credible lane.

Required lane names:

- `fast`: deterministic local proof under 2 minutes
- `contract`: public API/schema compatibility
- `db`: migrations, constraints, tenant/data rules
- `db-migration-analyze`: migration liability report (`jankurai migrate . --analyze --json target/jankurai/migration-report.json`); used when `agent/test-map.json` routes `db/migrations/` changes
- `web`: component/type/rendered UX behavior
- `e2e`: critical browser journeys
- `security`: secrets, dependencies, unsafe, SBOM/SCA
- `observability`: traces, request IDs, structured errors
- `audit`: jankurai JSON/Markdown report
- `release`: all merge gates

Tool replacement counts only when a Jankurai lane runs in CI and uploads the expected artifact evidence. Local config is readiness only; it does not count as replacement proof.

## Audit Output

Every audit should produce JSON and Markdown with:

- `standard_version`
- `auditor_version`
- `schema_version`
- `paper_edition`
- `target_stack_id`
- raw and final score
- hard caps
- dimension breakdown
- tool adoption readiness and replacement evidence
- findings with evidence
- ordered `agent_fix_queue`

## Repair Receipts

For non-trivial fixes, leave enough evidence for the next agent:

- changed paths
- failed rule or lane
- proof command
- artifact versions
- screenshot, crop, trace, ARIA snapshot, or audit report paths when UI or browser behavior changed
- remaining exception or follow-up

Operational receipts from `doctor`, `init`, and phase closeouts belong under `target/jankurai/receipts/` and should be cited by path when they matter.

## User-Provided Plans

When the user provides a paper, release, implementation, or handoff plan in the
conversation, treat that plan as controlling. Do not route it through local phase
or master-plan files unless the user explicitly names those local files. Before
broad validation, run `jankurai lane` or `jankurai proof` against changed paths
to choose the smallest credible proof lane. For audit requests, run
`cargo run -p jankurai -- . --json agent/repo-score.json --md agent/repo-score.md`.

## Local Commands

```bash
just versions
just fast
just score
just paper
just check
```

## v0.5 Daily Merge Loop

```bash
jankurai context-pack . --changed <path> --max-tokens 6000 --out target/jankurai/context-pack.json --md target/jankurai/context-pack.md
jankurai prove . --changed <path> --plan-out target/jankurai/proof-plan.json --plan-md target/jankurai/proof-plan.md
jankurai audit . --mode advisory --json target/jankurai/repo-score.json --md target/jankurai/repo-score.md
jankurai witness . --changed-from origin/main --baseline agent/repo-score.json --out target/jankurai/merge-witness.json --md target/jankurai/merge-witness.md
```

Ratchet mode requires an accepted baseline. Merge witness is the PR receipt:
no proof, no merge; no receipt, no trust.
