# Citation Index

| Claim ID | Claim | Status | Evidence / Basis |
| --- | --- | --- | --- |
| C-01 | Repository-level coding is meaningfully different from single-file completion. | Evidence-backed | SWE-bench, RepoBench, RepoExec |
| C-02 | Agent-computer interface design affects repository navigation, editing, and test execution. | Evidence-backed | SWE-agent |
| C-03 | Staged localization, repository graphs, and integration-width controls support better repository-level agent work. | Evidence-backed | Agentless, LocAgent, RepoGraph, RepoReason, Where Do AI Coding Agents Fail? |
| C-04 | Environment bootstrap is a distinct bottleneck for software-engineering agents. | Evidence-backed | SetupBench |
| C-05 | Agent evaluations should consider token and time resources, not only final pass rate. | Evidence-backed | SWE-Effi |
| C-06 | Functionally correct agent-generated code can still be insecure. | Evidence-backed | SUSVIBES |
| C-07 | Agents can over-retrieve and under-use context, so more context is not automatically better. | Evidence-backed | ContextBench and long-context bug-fixing work |
| C-08 | Repository-level context files can help or hurt depending on specificity and task fit. | Evidence-backed | Evaluating AGENTS.md and On the Impact of AGENTS.md Files |
| C-09 | Narrow, context-compatible skills are safer than broad skill packs that add token overhead. | Evidence-backed | SWE-Skills-Bench |
| C-10 | Human-workflow evidence cautions against assuming AI coding tools always speed expert developers. | Evidence-backed | METR productivity study and Professional Software Developers Don't Vibe, They Control |
| C-11 | Developer productivity should be treated as multidimensional rather than a single scalar metric. | Evidence-backed | SPACE framework |
| C-12 | Rust-SWE-bench provides direct Rust repository-level issue-resolution evidence and identifies Rust-specific difficulty around repository structure, types, traits, and reproduction. | Evidence-backed | Rust-SWE-bench/RUSTFORGER |
| C-13 | Rust API evolution and after-cutoff APIs remain difficult for LLM code generation. | Evidence-backed | RustEvo2 |
| C-14 | Safe and idiomatic repository-level C-to-Rust migration remains difficult. | Evidence-backed | CRUST-Bench and RustMap |
| C-15 | Compiler-guided LLM repair is a promising Rust-specific feedback loop. | Evidence-backed | RustAssistant |
| C-16 | LLM-assisted proof generation for Rust/Verus is a relevant high-assurance direction, not a general repository-agent solution. | Evidence-backed | AutoVerus |
| C-17 | Cargo workspaces and metadata expose machine-readable structure useful for agent maps. | Evidence-backed | Cargo workspaces and metadata docs |
| C-18 | Cargo `check` skips final code generation and is suitable for fast feedback. | Evidence-backed | Cargo check docs |
| C-19 | Cargo integration tests compile as separate targets, making test placement and target count consequential. | Evidence-backed | Cargo targets and Rust test organization docs |
| C-20 | Cargo features should be additive as a design policy, but repositories must test feature matrices because feature unification can expose hidden combinations. | Evidence-backed plus doctrine | Cargo features docs |
| C-21 | Rust API Guidelines support predictable, type-safe, dependable, debuggable, and future-proof public APIs. | Evidence-backed | Rust API Guidelines |
| C-22 | Google reports Android Rust changes with lower review/rollback burden and lower memory-safety vulnerability density than comparable C/C++ code. | Evidence-backed | Google Android security blog |
| C-23 | Cloudflare, Discord, Dropbox, AWS, Bottlerocket, Firecracker, and Hugging Face provide production-substrate evidence for Rust maturity. | Evidence-backed | First-party engineering posts and official project docs |
| C-24 | Production Rust adoption evidence should not be treated as direct proof of agent efficiency. | Proposed doctrine | Claim discipline policy |
| C-25 | Rust is not always token-cheapest, but it is a strong default when mistake cost dominates generation cost. | Proposed doctrine | Rust evidence, security evidence, language comparison |
| C-26 | Polyglot systems are acceptable when boundaries are generated, typed, and tested. | Proposed doctrine | Rust contract tooling and language-pairing analysis |
| C-27 | Minimum semantic surface should be framed as the smallest high-signal structure exposing owners, contracts, proof paths, token budgets, raw evidence, and widening rules. | Proposed doctrine | Synthesis of repository-agent, context, Cargo, and security evidence |
| C-28 | Short root AGENTS.md files should route rather than teach; detailed behavior belongs in local docs. | Proposed doctrine | AGENTS.md studies, OpenAI/Anthropic/GitHub guidance |
| C-29 | Code-size and file-size thresholds should be review triggers, not universal laws. | Proposed doctrine | Google small CL guidance, context evidence, Rust API Guidelines |
| C-30 | Recommended Rust function/file/patch/instruction budgets are house defaults for reducing token and locality cost. | Proposed doctrine | Operational design choice |
| C-31 | Generated maps should be first-class repo artifacts, not hidden retrieval traces. | Proposed doctrine | RIG, TDAD, Cargo metadata |
| C-32 | Newtypes, validated constructors, enums, private fields, and typed errors reduce hidden semantic surface. | Evidence-backed plus doctrine | Rust API Guidelines and error crate docs |
| C-33 | Libraries should normally expose typed errors and applications should attach rich context at boundaries. | Evidence-backed plus doctrine | thiserror, anyhow, miette docs |
| C-34 | Generated schemas and bindings reduce duplicated hand-maintained contracts. | Evidence-backed | Serde, SQLx, Schemars, Utoipa, ts-rs, Specta docs |
| C-35 | Structured tracing is especially useful for async and cross-layer debugging. | Evidence-backed | tracing docs |
| C-36 | Token savings are valid only when hidden correctness, security-gated validation, auditability, and human review do not regress. | Proposed doctrine | SWE-Effi, SUSVIBES, OWASP, RTK guardrails |
| C-37 | ETTS and SecureETTS must define per-attempt token cost, hidden-pass attempt, security-gated-pass attempt, retry cap, and included token classes explicitly. | Proposed doctrine | Measurement hygiene based on SWE-Effi and security evidence |
| C-38 | Raw logs saved but not shown to the model are audit evidence, not model-visible token cost. | Proposed doctrine | Token accounting policy |
| C-39 | `cargo check --message-format=json` and Cargo external-tool guidance support machine-readable Rust diagnostic compression. | Evidence-backed | Cargo check and external-tool docs |
| C-40 | RTK is a Rust CLI proxy for command-output filtering and token analytics with project-reported reductions. | Project claim | RTK repository and architecture docs |
| C-41 | RTK claims should remain project-reported until validated under hidden and security-gated criteria. | Proposed doctrine | Claim discipline policy |
| C-42 | Fast proof loops are central to agent efficiency because slow validation encourages speculative edits. | Proposed doctrine | SetupBench, SWE-Effi, Cargo docs, harness guidance |
| C-43 | cargo-nextest is suitable for fast and targeted Rust test execution. | Evidence-backed | cargo-nextest docs |
| C-44 | Rust testing ecosystem supports property, snapshot, compile-fail, fuzz, Miri, Loom, coverage, mutation, semver, and performance proof modes. | Evidence-backed | proptest, insta, trybuild, cargo-fuzz, Miri, Loom, cargo-llvm-cov, cargo-mutants, cargo-semver-checks, Criterion docs |
| C-45 | Build-loop acceleration should use targeted checks, stable target directories, measured compiler caches, Docker dependency caching, timings, and prebuilt tool installs where safe. | Evidence-backed plus doctrine | Cargo build cache/timings, sccache, cargo-chef, bacon, cargo-binstall docs |
| C-46 | Agent-generated tests require review and validation rather than being accepted as proof by default. | Evidence-backed | Rethinking Agent-Generated Tests |
| C-47 | Prompt injection, tool abuse, data exfiltration, excessive autonomy, cost denial, and supply-chain attacks are first-class agent security risks. | Evidence-backed | OWASP LLM Top 10 and AI Agent Security Cheat Sheet |
| C-48 | Secure-development and dependency-risk checks should be auditable rather than informal. | Evidence-backed | NIST SSDF and OpenSSF Scorecard |
| C-49 | Unsafe Rust should be surfaced explicitly and audited; unsafe-count tooling does not prove safety. | Evidence-backed plus doctrine | Rustonomicon, RustBelt, Rudra, cargo-geiger, Miri, sanitizers, Kani |
| C-50 | A compressed command summary is invalid if it hides panic, security, dependency, unsafe, or secret-exposure evidence. | Proposed doctrine | OWASP, RTK guardrails, token standard |
| C-51 | A Rust proof-cost catalog should include only tools that reduce owner ambiguity, proof cost, token noise, security risk, or contract drift. | Proposed doctrine | Minimum semantic surface doctrine |
| C-52 | cargo-public-api and cargo-semver-checks help expose public API drift. | Evidence-backed | Tool docs |
| C-53 | cargo-deny, cargo-audit, cargo-vet, cargo-about, and cargo-geiger belong in dependency/security lanes. | Evidence-backed | Tool docs |
| C-54 | rust-analyzer semantic tools can reduce broad file reads by answering definition, reference, and type questions directly. | Project/tool evidence | rust-analyzer MCP project docs and rust-analyzer ecosystem |
| C-55 | ARI-v0 should be a dashboard first and a single score second. | Proposed doctrine | SPACE, ContextBench, SWE-Effi, SetupBench, security evidence |
| C-56 | ARI-v0 should be capped or caveated when one-command setup, deterministic fast lane, security-gated validation, dependency/unsafe policy, or hidden validation is absent. | Proposed doctrine | SetupBench, SWE-Effi, SUSVIBES, OWASP |
| C-57 | `cargo-mss` / Rust Agent Rail Compiler is a high-impact untested concept for compiling repository constraints into an Agent Repair Manifest. | Proposed doctrine | Future-work synthesis from Cargo metadata, graph/localization work, and MSS doctrine |
| C-58 | ProofLens is a high-impact untested concept for proof-preserving Rust output compression. | Proposed doctrine | Future-work synthesis from RTK, Cargo JSON diagnostics, testing/security tooling, and token discipline |
| C-59 | `cargo-obligation-cache` is a high-impact untested concept for reducing repeated-proof tokens and conservative rerun cost. | Proposed doctrine | Future-work synthesis from proof-lane reuse, ETTS discipline, and conservative invalidation design |
| C-60 | Future systems must include build target, interface/artifact shape, metrics, and falsification criteria to avoid vaporware. | Proposed doctrine | V5 review discipline |
| C-61 | Task-aware context pruning can materially reduce coding-agent token use while preserving useful task context. | Evidence-backed | SWE-Pruner |
| C-62 | Readability and control-flow complexity work justify treating local cognitive burden as a review concern, even though this paper's numeric budgets remain doctrine. | Evidence-backed plus doctrine | Buse and Weimer, McCabe, Cognitive Complexity, Google small CL guidance |
| C-63 | Canonical repo-local command routers such as `just` reduce proof-lane ambiguity and narration overhead. | Project/tool evidence plus doctrine | just manual and proof-lane design |
| C-64 | Secret scanning, SBOM generation, vulnerability scanning, and workflow-security scanning belong in security-gated proof lanes when the repository ships binaries or uses CI/CD. | Evidence-backed plus project/tool evidence | OWASP, NIST SSDF, OpenSSF Scorecard, gitleaks, detect-secrets, Syft, Grype, actionlint, zizmor docs |
| C-65 | Polyglot boundary generators and packagers reduce handwritten contract drift when one source of truth is maintained. | Evidence-backed plus doctrine | Buf, OpenAPI TypeScript, Zod, PyO3, maturin, wasm-pack docs |
| C-66 | Non-Rust adjunct tooling should be treated as part of the proof surface when Rust repositories include web or desktop fronts. | Project/tool evidence plus doctrine | Biome, Vitest, Playwright, React, Tauri docs |
| C-67 | The largest token savings usually come from fewer wrong turns, narrower proof loops, and safer output shaping rather than shorter free-form prose alone. | Evidence-backed plus doctrine | ContextBench, SWE-Effi, SWE-Pruner, Anthropic context/tools guidance |
| C-68 | Low-token narration is acceptable only when exact commands, paths, error codes, failing test identities, advisories, seeds, exit codes, and raw-output handles remain intact. | Proposed doctrine | Token standard and harness guidance |
| C-69 | cargo-msrv and cargo-hakari are relevant proof-cost tools for MSRV discipline and large-workspace dependency unification. | Project/tool evidence | cargo-msrv and cargo-hakari docs |
| C-70 | cargo-auditable extends reviewable dependency state to produced binaries and SBOM workflows. | Project/tool evidence | cargo-auditable docs |
| C-71 | Project-reported semantic layers can reduce natural-language discovery and log parsing by exposing structured domain and operational state directly to agents. | Project claim | InsForge repository, docs, deployment post, and MCPMark post |
| C-72 | The strongest V7 future-work variants collapse into one deeper `cargo-mss` concept with negative-context compilation, capsule materialization, and active disambiguation as modes rather than separate flagship systems. | Proposed doctrine | V8 future-work synthesis from the V7 upgrade notes |
| C-73 | WarpOS local telemetry shows that runtime waste classes differ sharply by agent, with control-plane, catalog, analytics, and context-replay overhead varying enough to justify a staged runtime cascade rather than one generic filter. | Local measurement | WarpOS five-agent telemetry corpus and intervention catalog |
| C-74 | A paper-facing runtime taxonomy can collapse the engineering buckets into five stages: Suppress, Compress, Reuse, Steer, and Guard. | Proposed doctrine backed by local mechanism evidence | WarpOS intervention catalog and telemetry architecture |
| C-75 | WarpOS-style runtime interventions should be presented as bounded companion evidence for live waste visibility and intervention feasibility, not as proof of the global MSS thesis. | Proposed doctrine | Claim-discipline policy plus local WarpOS evidence family |
| C-76 | CrateAtlas-class repository graphs can serve as localization substrate for owner ranking, dependency/reference slicing, existence checks, and repair-capsule generation in runtime steering and guarding. | Project/tool evidence | CrateAtlas repository and API/model docs |
