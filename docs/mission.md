# Mission: Humans Were the Bug

Paper title: "Humans Were the Bug: From Vibe Coding to Agent-Native Engineering"

Jankurai exists to make agent-native engineering concrete. The claim is deliberately sharp: most repositories were shaped around human comfort, human memory, human navigation, and human tolerance for ambiguity. That design target is now wrong. Agents can generate code quickly, but they still fail when a repo hides ownership, accepts duplicate truth, lacks deterministic proof lanes, or forces the model to infer architecture from scattered convention.

The new standard is not "make coding pleasant." The new standard is "make wrong code easy to reject, localize, prove, audit, and repair."

The current working thesis for this repository is:

1. `jankurai init` installs the control plane.
2. `jankurai audit` and `jankurai doctor` make proof and drift visible.
3. The blessed stack is Rust core, TypeScript/React/Vite, PostgreSQL, generated contracts, and bounded Python.

Kafka deserves respect as proven brownfield streaming infrastructure, but it is not the stack identity. Jankurai should isolate event buses behind generated event contracts and Rust queue adapters, then evaluate Rust-native replacements as they earn compatibility and operations proof.

## Paper Mission

The paper argues that programming language history is a history of compression around the active bottleneck.

Machine code compressed nothing for people. Assembly gave names to operations. C compressed machine control into portable abstractions. Garbage-collected languages compressed memory management into runtime policy. Object-oriented languages compressed team-scale modeling into classes and interfaces. Scripting languages compressed deployment friction and iteration time. TypeScript compressed JavaScript's social and production chaos into a typed product surface. Rust compressed memory discipline, concurrency control, and invariant preservation into a compiler-enforced bargain.

That history matters because the bottleneck moved again. The hard part is no longer whether a human knows the syntax. Agents can learn syntax. The hard part is whether the codebase gives the agent short, enforceable paths from intent to proof. A language, framework, or repo layout wins in the AI era only when it lowers the cost of rejecting bad generated work.

The paper must keep this through-line:

| Era | Dominant bottleneck | Winning compression |
| --- | --- | --- |
| Early machine era | hardware control | opcodes, registers, direct memory |
| Assembly and C era | portability and systems control | symbolic names, structs, compilation, ABI discipline |
| OOP and VM era | large human teams | encapsulation, runtime portability, managed memory |
| Web and scripting era | product iteration | dynamic runtimes, package ecosystems, fast feedback |
| Typed product era | production web scale | TypeScript, typed APIs, build tooling, component systems |
| Agent era | verification and repair | strict ownership, generated contracts, proof lanes, audit evidence |

The paper should not pretend every language hype cycle was foolish. It should separate technical promise from adoption reality.

## Dead Hype And Stalled Promise

jankurai needs a serious subsection on languages and platforms that were technically interesting but failed to become the universal answer. This is not a dunk list. It is a warning against confusing elegance with ecosystem victory.

Required angle:

| Language or platform | Real promise | Adoption friction |
| --- | --- | --- |
| Julia | scientific productivity, multiple dispatch, JIT performance, MATLAB-like ergonomics | package maturity gaps, latency, deployment friction, smaller industrial base |
| Scala | expressive type system, JVM leverage, functional and object-oriented blend | complexity, slow builds, style fragmentation, hiring friction |
| Haskell | purity, types, equational reasoning | steep learning curve, production ecosystem limits, integration cost |
| Erlang and Elixir | concurrency, supervision, fault tolerance, realtime systems | narrower fit outside telecom/realtime/collaboration domains |
| Clojure | data-oriented design, Lisp power, JVM host | syntax barrier, smaller hiring pool, runtime and tooling expectations |
| F# | strong functional language on .NET | overshadowed by C#, smaller ecosystem and mindshare |
| D, Nim, Crystal, Zig | systems/productivity blends with strong ideas | ecosystem depth, stability expectations, corporate backing, migration cost |
| Elm, ReScript, ReasonML | safer frontends and strong compile-time guarantees | React/TypeScript gravity, ecosystem churn, migration friction |

The lesson: technical promise is not enough. In the agent era, the same rule gets harsher. A stack must provide proof speed, contract discipline, security posture, tool depth, hiring surface, and corpus strength. Being elegant is not enough.

## Winner Doctrine

jankurai chooses one default stack:

```text
Rust core + TypeScript/React/Vite product surface + PostgreSQL truth
+ generated contracts + bounded Python AI/data service
```

The second half of the paper should care about the winner only. Once the ranking is argued, the practical question changes from "what else could be nice?" to "how do we build repositories where agents stop guessing?"

The winner stack exists because each piece owns a different kind of truth:

| Layer | Owns | Does not own |
| --- | --- | --- |
| TypeScript/React/Vite | UI, forms, client state, generated API clients, fast product proof | secrets, durable product truth, direct DB writes, core authorization |
| Rust API edge | request validation, authz, rate limits, idempotency, orchestration | UI behavior, raw product experiments, scattered SQL |
| Rust domain | IDs, invariants, state machines, pure decisions, stable error codes | I/O, environment reads, network calls, framework code |
| Rust application | commands, queries, transaction scope, use-case policy | transport details, direct UI state, adapter implementation |
| Rust adapters | DB, queues, HTTP clients, filesystem, secrets, external APIs | domain rules |
| PostgreSQL | durable truth, constraints, migrations, indexes, RLS where useful | hidden app-only invariants |
| Python AI service | inference, embeddings, eval harnesses, prompt/data tooling | authz, billing, product truth, direct production DB ownership |
| Ops/security | CI lanes, OTel, SBOM, SCA, secret scanning, provenance | hidden feature logic |

The standard is intentionally narrow. A narrow standard is auditable. A broad preference list is not.

## Rubric Mission

The paper rubric must be the central argumentative device. It should be a real scorecard, not a decorative table.

| Criterion | Weight | Argument |
| --- | ---: | --- |
| Agent-verifiable correctness loop | 18 | The stack must reject wrong output quickly through compilers, generated contracts, deterministic tests, and small proof surfaces. |
| Security, memory safety, supply chain | 16 | AI increases the volume of plausible but vulnerable code. Memory safety, dependency controls, secret scanning, and reviewable unsafe zones must dominate preference. |
| Runtime performance, memory, cloud cost | 13 | Agent-era products use more background work, inference calls, queues, and automation. Waste becomes infrastructure tax. |
| Concurrency and resilience | 12 | Agents create parallel work, cancellation paths, retries, and partial failures. The core stack must make backpressure and recovery explicit. |
| Boundary and contract integrity | 11 | Polyglot systems only work when contracts are generated, tested, and owned. Handwritten mirrors are drift. |
| Simplicity and review surface | 10 | Cheap code generation makes code volume less impressive. Reviewable behavior matters. |
| Ecosystem, hiring, AI corpus strength | 8 | Practicality matters, but cannot outrank correctness and security. |
| Observability, auditability, provenance | 6 | Bad generated code must leave traces that humans and agents can use to repair production. |
| Data integrity and durable workflow fit | 4 | PostgreSQL constraints, migrations, indexes, and workflow durability protect truth when app code is wrong. |
| Product velocity and interop | 2 | Velocity matters only after safety and proof are in place. |

The graph ranking should make the point visually: Rust/TypeScript/PostgreSQL wins by proof and safety, Go is the practical default, .NET wins regulated enterprise contexts, TypeScript-plus-compute-cells wins product velocity with boundary risk, and JVM modernization wins where the JVM estate is already real.

## Audit Vision

The jankurai audit is the enforcement layer for the paper.

It must run in every CI pipeline. It must fail fast. It must produce two outputs: a machine-readable contract and a human-readable repair brief. The audit is not allowed to merely complain. Every finding must include:

| Field | Purpose |
| --- | --- |
| `severity` | make triage deterministic |
| `category` | route ownership |
| `path` | remove ambiguity |
| `problem` | state the violation |
| `agent_fix` | give the next actionable repair |
| `evidence` | show concrete scan hits |
| `standard_version` | bind result to a versioned rule set |
| `auditor_version` | bind result to an implementation release |
| `schema_version` | bind result to output compatibility |
| `paper_edition` | bind result to the paper argument in force |
| `target_stack_id` | identify the scoring target without prose parsing |

The audit should judge only the chosen stack. It is not a generic linter. It should reward Rust core, TypeScript product surface, PostgreSQL truth, generated contracts, bounded Python, and proof evidence. It should call out unnecessary Python, unbounded polyglot sprawl, direct DB access from the UI, handwritten contract mirrors, missing generated zones, and framework-driven business logic.

## Vibe Coding Violations

"Vibe coding" means code accepted because it appeared plausible, not because the repo made it provable. The audit should treat these as hard violations when evidence is present:

| Violation | Why it is dangerous | Expected repair |
| --- | --- | --- |
| Duplicate business logic | agents patch one copy and miss another | move rule into Rust domain or generated contract |
| Handwritten DTO mirrors | API drift hides until runtime | generate clients/types from OpenAPI, Protobuf, or JSON Schema |
| Fallback cascades | wrong behavior gets masked | replace with explicit error and documented recovery |
| Silent catch-all exceptions | failures become unverifiable | throw typed agent-friendly exceptions with docs links |
| Mega files | agents lose locality and edit too broadly | split by ownership and proof lane |
| Mega functions | behavior cannot be localized | refactor into named commands, policies, validators |
| Junk-drawer directories | `utils`, `helpers`, `common`, and `misc` become architecture dumps | create named modules by domain purpose |
| Hidden I/O in core | pure code becomes untestable | move I/O to adapters |
| Direct DB access from UI or Python | product truth escapes ownership | route through Rust API/application boundary |
| Business rules in adapters | persistence details become policy | move rules to domain/application |
| Scattered raw SQL | schema coupling becomes invisible | centralize in adapters and migration-backed queries |
| Unbounded Python | dynamic code owns too much truth | box Python under `python/ai-service` with typed contracts |
| Untested generated code drift | generated files become stale caches | require generator source and diff checks |
| Secret-bearing examples | agents copy unsafe patterns | scrub examples, add secret scanning |
| TODO-driven design | ambiguity becomes backlog-shaped production logic | require owner, date, issue, and exit condition |
| Optional validation lanes | agents skip proof under pressure | make fast lane deterministic and CI-enforced |
| Inconsistent naming | retrieval and grep become unreliable | enforce domain vocabulary and suffix rules |
| Conflicting agent instructions | models choose randomly | keep root guidance short and path rules specific |
| Oversized instruction files | token budget burns before work starts | split into routed local docs and concise root files |
| Undocumented exceptions | teams relearn old failures | add exception catalog entries with fixes and docs |

Hard line: if a pattern makes agent repair slower, less local, or less provable, it belongs in the audit.

## Agent-Friendly Exceptions

Agent-friendly exceptions are a major jankurai idea. They turn repeated failure knowledge into code and documentation, not tribal memory.

Every intentional exception class or error enum variant should carry:

| Required field | Meaning |
| --- | --- |
| `name` | stable identifier agents can search |
| `purpose` | what invariant or boundary it protects |
| `reason` | why this instance was thrown |
| `common_fixes` | short list of likely repairs |
| `docs_url` | direct link to local docs or public standard |
| `owner` | owning cell |
| `severity` | repair urgency |
| `retryable` | whether automation may retry |
| `evidence_id` | log/trace/request id when runtime-raised |

Language guidance:

| Language | Preferred shape |
| --- | --- |
| Rust | typed error enum, stable codes, `thiserror`/custom display, structured context, no stringly business errors |
| TypeScript | discriminated union for expected failures, typed `Error` subclass only for exceptional runtime failure, generated API error types |
| PostgreSQL | named constraints and SQLSTATE-aware mapping to Rust domain/application errors |
| Python AI service | typed exception classes at the service boundary only, converted into schema-defined API errors |

The goal is not to throw more chaos. The goal is to make every known failure teach the next agent what happened, why, and how teams usually fix it.

## Test Coverage Mission

Agent-native testing is not "more tests everywhere." It is routed proof.

| Layer | Required proof |
| --- | --- |
| Rust domain | unit tests, property tests for invariants, table tests for state machines |
| Rust application | command/query tests, authz tests, idempotency tests, transaction behavior |
| Rust adapters | integration tests against real or faithful local services, migration-backed DB tests |
| TypeScript UI | component tests, contract-backed client tests, accessibility checks |
| End-to-end | Playwright or equivalent browser tests for critical product flows |
| PostgreSQL | migration tests, constraint tests, schema drift checks, rollback checks |
| Python AI service | contract tests, eval fixtures, regression cases for prompts/models/data transforms |
| Ops/security | secret scanning, dependency scanning, SBOM/provenance, unsafe/dependency rationale |

The CI layout should have at least:

| Lane | Purpose |
| --- | --- |
| `fast` | deterministic local gate for most agent edits |
| `contracts` | generated API/schema drift |
| `security` | secrets, dependencies, SBOM/SCA, unsafe ledger |
| `db` | migrations, constraints, schema drift |
| `ui` | component and browser smoke checks |
| `full` | slower integration and E2E suite |
| `audit` | jankurai score and hard-rule findings |

The audit should punish missing lanes because missing proof is not neutral. It is permission for bad generated code to survive.

## Agent Tooling Standard

Different agents load instructions differently. The standard should support them without turning the repo into a prompt junkyard.

| Tool family | Standard artifact |
| --- | --- |
| Codex | root `AGENTS.md`, scoped `AGENTS.override.md`, explicit validation commands |
| Claude Code | `CLAUDE.md` or `.claude/CLAUDE.md`, scoped `.claude/rules/`, concise memory discipline |
| Cursor | `.cursor/rules` with path-scoped rules and concrete project patterns |
| GitHub Copilot | `.github/copilot-instructions.md` and `.github/instructions/*.instructions.md` |
| Antigravity-style agent IDEs | mission/control docs, artifact expectations, terminal/browser approval rules |
| Aider-style CLI agents | repo maps, symbol maps, token budgets, precise file ownership |

Common rule: root instructions must route, not teach everything. Official Claude docs recommend concise project instructions and target under 200 lines per `CLAUDE.md`; GitHub Copilot docs say repository instructions should be short and broadly applicable; OpenAI Codex docs define hierarchical `AGENTS.md` loading and a default 32 KiB project-doc limit; Aider's repo map shows the value of compact symbolic context. These all point to the same standard: small root, local detail, generated maps, no contradiction.

## Token Economy

Token minimization is an engineering concern, not a personality preference.

Generally accepted practices:

| Practice | Why it helps |
| --- | --- |
| short root instructions | preserve context for the task |
| path-scoped rules | load specialized detail only when needed |
| owner maps and test maps | avoid broad exploration |
| generated repo maps | provide symbols without dumping files |
| stable naming | make search cheaper and less ambiguous |
| hard file/function limits | keep context slices small |
| machine-readable audit output | let agents repair without reading prose reports |
| RTK-style filtered command output | reduce noisy shell output when filtering is safe |
| proof receipts | preserve exact commands and outcomes without rerunning broad scans |

Compressed writing styles can help in personal workflows, but they should not become the standard unless they remain readable, auditable, and acceptable to the team. The standard should optimize evidence density, not novelty.

## Versioned Standard

jankurai must version three things separately:

| Artifact | Versioned as | Reason |
| --- | --- | --- |
| paper | edition | argument and evidence will evolve |
| standard | semantic version | repos need stable compliance targets |
| audit script | semantic version plus schema version | CI needs compatibility guarantees |

Every audit result should include `standard_version`, `auditor_version`, `schema_version`, `paper_edition`, and `target_stack_id`. Repos should pin a target version and check for newer releases. Breaking rule changes require a major version. New advisory checks can ship in minor versions. Copy edits and documentation clarifications can ship as patches.

## Public Evidence Base

Source tiers:

| Tier | Examples | How to use |
| --- | --- | --- |
| Primary/official | GitHub Octoverse, NSA/CISA, DORA, Veracode, GitGuardian, OpenTelemetry, OpenAI Codex docs, Claude Code docs, GitHub Copilot docs | factual claims and hard recommendations |
| Project documentation | Aider repo map, Vite, TypeScript, language docs, Google Antigravity codelab | tool behavior and project claims |
| Research preprints | SWE-PolyBench, AGENTS.md/context-file evaluations, Claude manifest studies | emerging evidence with caveats |
| Community sources | Reddit, X, blog writeups, issue threads | low-weight signals for pain points and adoption friction |

References already carried by the paper:

- GitHub Octoverse 2025: TypeScript growth and AI development signals.
- NSA/CISA memory-safe language guidance: memory safety as policy-backed security direction.
- DORA 2025: AI amplifies organizational strengths and weaknesses.
- Veracode 2025: generated code can be syntactically correct and still insecure.
- GitGuardian 2026: secret sprawl and AI-assisted leakage risk.
- OpenTelemetry: vendor-neutral traces, metrics, and logs.

Agent-specific sources to keep in view:

- OpenAI Codex `AGENTS.md` docs: https://developers.openai.com/codex/guides/agents-md
- Claude Code memory docs: https://code.claude.com/docs/en/memory
- GitHub Copilot custom instructions: https://docs.github.com/en/copilot/concepts/prompting/response-customization
- Aider repo map: https://aider.chat/docs/repomap.html
- Google Antigravity codelab: https://codelabs.developers.google.com/getting-started-google-antigravity
- SWE-PolyBench: https://arxiv.org/abs/2504.08703
- Evaluating `AGENTS.md`: https://arxiv.org/abs/2602.11988
- Claude manifest study: https://arxiv.org/abs/2509.14744

Important caveat: emerging research is mixed. Some studies suggest repository context files can increase exploration cost or reduce task success when they add unnecessary requirements. jankurai should respond by making instructions smaller, more specific, more local, and auditable, not by abandoning repo guidance.

## Future Research

The standard needs measured proof, not just conviction.

Research agenda:

| Question | Needed evidence |
| --- | --- |
| Do jankurai repos reduce wrong-owner edits? | controlled agent tasks before/after owner maps and proof lanes |
| Do generated contracts reduce repair time? | drift-injection experiments across UI/API/Rust boundaries |
| What file/function limits best improve agent repair? | benchmark edit locality, token use, and regression rate |
| Which exception shapes help agents self-repair? | compare structured exceptions against plain errors |
| How much Python is too much? | measure incidents and repair cost when Python crosses product-truth boundaries |
| Which instruction formats work across tools? | Codex, Claude, Cursor, Copilot, Antigravity-style agent IDEs |
| Can audit scores predict production reliability? | longitudinal CI score vs incident and rollback data |
| How do token budgets affect correctness? | vary instruction size, repo-map size, and local docs |

## Known Gaps

Known gaps must stay explicit:

- The stack ranking is a doctrine backed by evidence, not a universal benchmark.
- Public agent-tool behavior changes quickly; tool-specific guidance must be refreshed often.
- Community reports from X, Reddit, and blogs are useful for signals but weak as proof.
- The audit can detect structural risk, not semantic correctness.
- File-size and function-size limits need empirical calibration by language and domain.
- Generated contracts help boundary drift but do not prove business correctness.
- Python containment is a strong default, but some data-heavy companies will need explicit exception rules.
- Playwright-style browser testing is strong for product flows, but it can become slow and flaky without strict test ownership.
- Agent-friendly exceptions are promising but need conventions, libraries, and evidence from real repairs.
- The standard currently optimizes one stack. That is the point, but it limits generality.

## Adoption Mission

jankurai should become the agent-native engineering standard by being useful before it is famous.

The adoption path is:

1. Publish the paper as the argument.
2. Publish the audit as the enforcement layer.
3. Publish templates that make compliant repos easier to start than noncompliant repos.
4. Publish CI integrations that make drift visible on every pull request.
5. Publish public audit reports on representative open-source repos.
6. Publish repair playbooks that agents can execute.
7. Publish versioned rule packs for Codex, Claude, Cursor, Copilot, and agent-first IDEs.
8. Publish benchmark results showing repair speed, token use, wrong-owner edits, and regression rate.

Strong claim: human-first repositories are legacy infrastructure. Agent-native repositories are the next professional baseline. The teams that adapt will compound. The teams that keep accepting plausible code without proof will drown in generated debt.
