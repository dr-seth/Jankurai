# humanlint Audit Rubric

Version: `0.2.0`

Target stack: Rust core + TypeScript/React/Vite product surface + PostgreSQL truth + generated contracts + bounded Python AI/data service.

The audit is strict on purpose. It is not a general-purpose repo quality score. It asks one question: can an agent safely reject, localize, prove, audit, and repair this codebase without turning human-friendly shortcuts into production behavior?

## Required Shape

| Surface | Allowed Role | Hard Boundary |
| --- | --- | --- |
| `apps/web/` | TypeScript, React, Vite, generated API clients, UI state, forms, local validation | no secrets, no direct DB, no handwritten DTO drift, no durable truth |
| `apps/api/` | Rust HTTP/RPC edge, request decoding, response encoding, auth/session bridge | no raw SQL in handlers, no domain rules hidden in framework code |
| `crates/domain/` | pure IDs, invariants, state machines, decisions, typed errors | no I/O, env, DB, HTTP, filesystem, queues, logging side effects |
| `crates/application/` | commands, authz, idempotency, transactions, workflow orchestration | no UI concerns, no scattered raw SQL, no provider-specific details |
| `crates/adapters/` | PostgreSQL, queues, external APIs, filesystem, env, providers | no domain rules |
| `crates/workers/` | async jobs, durable workflow glue, CPU workers | no UI truth, no bypass of application/domain invariants |
| `contracts/` | OpenAPI, protobuf, JSON Schema, generated contract outputs | generated outputs must be marked and repaired from source contracts |
| `db/` | migrations, constraints, seeds, indexes, RLS where useful | no ad hoc app-only durable invariants |
| `python/ai-service/` | models, embeddings, evals, notebooks, typed model/data API | no product truth, no core authz, no direct production DB ownership |
| `ops/` | CI, observability, security, provenance, deployment | no hidden manual gates |

## Score Dimensions

| Dimension | Weight | What Good Looks Like |
| --- | ---: | --- |
| Ownership and navigation surface | 14 | root `AGENTS.md`, local routing docs, owner map, test map, short navigation |
| Contract and boundary integrity | 14 | generated clients, checked API drift, strict TypeScript, Rust typed boundaries |
| Proof lanes and test routing | 14 | one-command validation, deterministic fast lane, CI audit lane, e2e/property/integration tests |
| Security and supply-chain posture | 14 | lockfiles, secret scanning, dependency review, SBOM/provenance, workflow linting |
| Code shape and semantic surface | 12 | small files/functions, low duplication, no placeholder/fallback behavior, specific names |
| Data truth and workflow safety | 8 | migrations, constraints, DB isolated to adapters/db, no DB writes from wrong layers |
| Observability and repair evidence | 8 | tracing, request IDs, structured diagnostics, repair receipts, agent-friendly exceptions |
| Context economy and agent instructions | 8 | concise docs, generated zones, root router, evidence paths, no token-heavy maze |
| Python containment and polyglot hygiene | 4 | Python only in bounded AI/data or tooling, minimal runtime share, no unnecessary languages |
| Build speed signals | 4 | fast checks, caching, nextest/vitest, targeted commands, locked dependencies |

## Hard Rule Caps

| Rule | Max Score | Agent Repair |
| --- | ---: | --- |
| no root agent/developer instructions | 75 | add concise root `AGENTS.md` and route deeper docs locally |
| no one-command setup or validation | 70 | add canonical `setup`, `check`, `test`, or `verify` command |
| no deterministic fast lane | 65 | add the narrowest repeatable proof loop for changed files |
| high-risk repo with no security lane | 60 | add secret scan, dependency review, SBOM/provenance, workflow lint |
| generated contracts or public API drift untested | 80 | generate clients and gate drift in CI |
| Python owns product truth or DB ownership | 72 | move truth/authz/workflows into Rust and DB migrations |
| no secret or dependency scan in CI | 78 | add gitleaks/detect-secrets plus dependency review or equivalent |
| no humanlint audit lane in CI | 82 | run `tools/humanlint.py` in every PR and publish JSON/Markdown |
| non-optimal product language found | 74 | migrate product runtime code to Rust, TypeScript, SQL, contracts, or bounded Python |
| too much Python in product surface | 72 | box Python into model/data service and move durable behavior to Rust |
| vibe placeholders in product code | 68 | replace TODO/stub/unimplemented/unreachable with real behavior or typed exceptions |
| fallback soup in product code | 70 | replace fallback chains with explicit states, bounded retries, telemetry, docs |
| future-hostile/dead-language in product runtime code | 64 | remove or rename dead/temporary/legacy wording, implement the state, or move copy/docs/generated/vendor text into an allowlisted context |
| severe duplication in product code | 70 | extract one named boundary and test it before editing behavior |
| generated zone mutation risk | 76 | add generated zone manifest and repair generated files from source contracts |
| direct DB access from wrong layer | 66 | move SQL and DB clients to `crates/adapters` or `db/` |
| missing web e2e lane | 82 | add Playwright or equivalent e2e tests for critical user flows |
| missing Rust property/integration tests | 82 | add invariant/property tests plus integration tests through cargo test/nextest |
| no agent-friendly exception pattern | 76 | add typed errors with code, purpose, reason, common fixes, docs URL |
| missing agent-readable docs | 80 | add concise architecture, boundary, testing, and audit docs |

## Known Vibe-Coding Insults

These are hard repair signals, not style nits.

| Insult | Why It Fails Agent-Native Engineering | Required Repair |
| --- | --- | --- |
| duplicated logic | agents patch one copy and miss another | extract one owned module and add tests |
| fallback soup | behavior becomes probabilistic and unreviewable | model explicit states and bounded retry policy |
| future-hostile/dead language | `legacy`, `deprecated`, `old`, `temporary`, `workaround`, `shim`, `fallback`, `TODO`, and similar markers train agents to preserve abandoned paths | delete, rename, implement, or move quoted product copy/docs/generated/vendor text into an allowlisted context |
| TODO/FIXME/HACK/XXX | placeholder intent becomes shipped behavior | implement or create typed unsupported-state exception |
| stub/placeholder/not implemented | fake completeness blocks proof | delete, implement, or gate behind explicit exception |
| `unreachable!`, `unimplemented!`, TODO panics | runtime surprise hidden from proof lanes | replace with typed errors and tests |
| handwritten DTOs | frontend/backend drift silently | generate from OpenAPI/protobuf/JSON Schema |
| handwritten fetch wrappers | every endpoint becomes a local contract fork | generate API client and keep one transport wrapper |
| direct DB from UI/API/domain/application | durable truth leaks into wrong layer | isolate DB in `crates/adapters` and `db/` |
| Python product truth | dynamic runtime owns durable business state | move truth/authz/workflows into Rust/PostgreSQL |
| unnecessary runtime languages | more syntax, tooling, locks, and failure modes | converge product runtime to the target stack |
| mega files | agents lose locality and reviewers lose ownership | split before 500 LOC, prefer under 300 LOC |
| mega functions | behavior cannot be named, tested, or localized | keep under 80 LOC by default |
| weak names | ownership and intent are hidden | use domain verbs/nouns, not `utils`, `helpers`, `manager`, `common` |
| missing docs | agents infer policy from code accidents | add short routed docs and local ownership files |
| missing audit CI lane | rules are advisory instead of enforced | run audit in every PR |
| mutated generated zones | generated code becomes forked source | edit source contract, regenerate, verify |
| no e2e web proof | UI regressions depend on human clicking | add Playwright critical-path tests |
| no Rust property tests | invariants are example-only | add `proptest`/equivalent invariant tests |
| no Rust integration tests | cross-crate behavior is unproved | add tests under crate or workspace `tests/` |
| no security scan | AI-churned dependencies and secrets slip through | run secret/dependency/provenance gates |
| opaque exceptions | failures tell humans too little and agents nothing | standardize agent-friendly exceptions |
| console/println debugging | production evidence is unstructured | use tracing, request IDs, and structured logs |
| junk drawer folders | every patch becomes global search | replace with owned domain/adapters modules |

## Future-Hostile Language Rule

Product/runtime code must not contain future-hostile or dead-language markers such as `legacy`, `deprecated`, `depricated`, `obsolete`, `old`, `temporary`, `temp`, `workaround`, `shim`, `compat`, `backcompat`, `fallback`, `best effort`, `cleanup later`, `remove later`, `dead code`, `unused`, `stale`, `hack`, `todo`, `fixme`, `placeholder`, `stub`, or `dummy`.

The rule is strict because these words encode uncertainty as production behavior. An agent should not infer whether a `legacy` branch is still required, whether a `temporary` path can be removed, or whether a `fallback` is intentional policy.

Allowlisted contexts are path-based and must be obvious: documentation, reference material, generated files, vendor code, or explicitly named product-copy surfaces such as `product-copy`, `copydeck`, `i18n`, `locales`, or `translations`. The allowlist is not a comment escape hatch. If the file owns runtime behavior, repair the term by naming the real state, implementing the behavior, or raising a typed unsupported-state exception.

Every finding for this rule must include `path`, `line`, `matched_term`, `reason`, and `agent_fix` so agents can patch exact evidence without broad searching.

## Agent-Friendly Exceptions

Every controlled error that can reach logs, API responses, background jobs, or tests should expose:

| Field | Required Meaning |
| --- | --- |
| `name` | stable exception or error name |
| `code` | stable machine-readable code |
| `purpose` | what invariant or boundary this error protects |
| `reason` | why this instance failed |
| `common_fixes` | concrete repair candidates for agents and humans |
| `docs_url` | direct local or public documentation link |
| `source` | underlying provider/system error, when safe |
| `correlation_id` | trace/request/job id for production repair |

Rust should prefer enum error types with `thiserror` or equivalent plus structured diagnostic fields. TypeScript should mirror boundary errors with `Error` subclasses or discriminated result unions. Python AI/data services should return typed API errors, not raw provider exceptions.

## Test Standard

| Layer | Minimum Proof |
| --- | --- |
| Rust domain | unit tests plus property tests for invariants/state machines |
| Rust application | integration tests for authz, idempotency, transactions, workflows |
| Rust adapters | DB integration tests, migration tests, external API contract tests or fakes |
| TypeScript web | unit/component tests for pure UI logic plus Playwright e2e critical paths |
| Contracts | generation test, drift check, schema compatibility check |
| PostgreSQL | migration apply/rollback where possible, constraint tests, seed validation |
| Python AI/data | eval tests, contract tests, no product-truth tests that imply ownership |
| Ops/security | secret scan, dependency/SBOM scan, workflow lint, audit scorer in CI |

## CI Contract

Every repository adopting this standard should run:

```bash
python3 tools/humanlint.py . --json repo-score.json --md repo-score.md
```

The JSON is the machine contract. The Markdown is the review surface. CI should upload both artifacts and fail when score or hard-cap policy crosses the team threshold.

The output must include:

| Field | Purpose |
| --- | --- |
| `standard_version` | lets repos track standard upgrades |
| `target_stack` | prevents generic scoring drift |
| `score` and `raw_score` | final capped score plus weighted score |
| `caps_applied` | hard rule failures |
| `dimensions` | weighted breakdown |
| `findings` | actionable evidence with path, line, matched term, reason, problem, and repair |
| `agent_fix_queue` | ordered repair work for coding agents |

## Versioning

The audit script, paper, and agent-facing artifacts must version together. Repos should record the humanlint standard version they target and schedule regular checks for newer releases. Breaking audit changes should include migration notes and example repairs.
