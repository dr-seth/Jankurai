# Rust Proof-Cost Tooling Catalog

Status: supplement artifact for the V7 review-backed standard.

This catalog is intentionally role-based rather than popularity-based. A tool belongs here only if it reduces at least one of:

- owner ambiguity
- contract drift
- proof cost
- token noise
- security ambiguity
- review reconstruction cost

## Install Profiles

### Minimal profile

Use this when the repository is mostly Rust and the goal is a fast, reviewable inner loop.

- `cargo-nextest`
- `cargo-hack`
- `cargo-semver-checks`
- `cargo-deny`
- `cargo-audit`
- `cargo-vet`
- `cargo-geiger`
- `cargo-llvm-cov`
- `rust-analyzer`
- `just`
- `bacon`

### Recommended profile

Add this when the repository has public contracts, richer CI, or multiple execution surfaces.

- `cargo-public-api`
- `cargo-msrv`
- `cargo-udeps`
- `cargo-machete`
- `cargo-about`
- `cargo-auditable`
- `cargo-mutants`
- `cargo-fuzz`
- `Miri`
- `Loom`
- `Kani`
- `Criterion`
- `cargo-limit`
- `ast-grep`
- `cross`
- `sccache`

### High-assurance or supply-chain profile

Add this when the repository ships binaries, has unsafe/FFI, or runs privileged CI/CD.

- `gitleaks`
- `detect-secrets`
- `Syft`
- `Grype`
- `zizmor`
- `actionlint`

### Boundary-heavy or polyglot profile

Add this when the repository crosses API, protobuf, Python, or WebAssembly boundaries.

- `Buf`
- `tonic`
- `prost`
- `openapi-typescript`
- `Zod`
- `PyO3`
- `maturin`
- `wasm-pack`

### Web and desktop adjuncts

Use these on the non-Rust side of Rust-centered systems.

- `Biome`
- `Vitest`
- `Playwright`
- `React`
- `Tauri`
- `Ratatui`
- `Leptos`
- `Dioxus`

## Catalog

| Role | Tool | What It Reduces | Use When | Evidence Class | Caveat |
| --- | --- | --- | --- | --- | --- |
| Navigation/localization | `cargo metadata --no-deps --format-version 1` | blind repo traversal | owner unclear | official Cargo docs | graph only, not semantic ownership |
| Navigation/localization | `rust-analyzer` | broad file reads | type, def, ref, impl questions | project/tool docs | editor/LSP integration quality varies |
| Navigation/localization | `ast-grep` | noisy text search | structural search, codemods, rule checks | project docs | language grammar support matters |
| Navigation/localization | `cargo-limit` | over-large diagnostic reads | large compiler error sets | project docs | shaping aid, not a proof tool |
| Command routing | `just` | repeated command narration | canonical proof lanes and setup | project docs | command names still need governance |
| Fast proof | `cargo-nextest` | slow or noisy test loop | targeted and workspace tests | project docs | still requires sane test ownership |
| Feature/API hygiene | `cargo-hack` | feature-matrix ambiguity | additive feature policy and CI | project docs | can become expensive if the matrix is unbounded |
| Feature/API hygiene | `cargo-semver-checks` | hidden public API drift | published crates and shared libraries | project docs | public API policy must be defined |
| Feature/API hygiene | `cargo-public-api` | public-surface ambiguity | library crates | project docs | complements semver checks, does not replace them |
| Dependency hygiene | `cargo-udeps` | dependency noise | stale or accidental dependencies | project docs | false positives possible in unusual builds |
| Dependency hygiene | `cargo-machete` | manifest clutter | dependency cleanup | project docs | review before automated removals |
| MSRV discipline | `cargo-msrv` | hidden compiler-version drift | library or toolchain contracts matter | project docs | true MSRV can still depend on dependency behavior |
| Workspace build shaping | `cargo-hakari` | redundant workspace dependency builds | large workspaces with shared deps | project docs | most useful at scale |
| Contracts/boundaries | `Serde` | handwritten serialization drift | JSON/TOML/YAML data paths | project docs | schema discipline still required |
| Contracts/boundaries | `SQLx` | unchecked SQL drift | DB-heavy services | project docs | compile-time checks depend on setup mode |
| Contracts/boundaries | `Schemars` | handwritten JSON schema drift | schema publishing or validation | project docs | decide which schema is source of truth |
| Contracts/boundaries | `Utoipa` | handwritten OpenAPI drift | HTTP APIs | project docs | API style still needs review |
| Contracts/boundaries | `ts-rs` | Rust-to-TypeScript drift | shared internal types | project docs | best when Rust stays source of truth |
| Contracts/boundaries | `Specta` | cross-language type drift | Tauri or TS bindings | project docs | boundary model still needs policy |
| Contracts/boundaries | `Buf` | protobuf plugin/version drift | protobuf/gRPC repos | official/project docs | schema governance must still be explicit |
| Contracts/boundaries | `tonic` / `prost` | manual gRPC/wire boilerplate | Rust gRPC services | project/docs.rs | generated code should not be edited |
| Contracts/boundaries | `openapi-typescript` | handwritten TS contract drift | HTTP boundaries with TS clients | project docs | pair with runtime validation when needed |
| Contracts/boundaries | `Zod` | TS runtime contract ambiguity | client/runtime validation | project docs | runtime schema can drift if not generated |
| Polyglot boundary | `PyO3` | ad hoc Python FFI | Rust core plus Python edge | project docs | boundary ownership must stay narrow |
| Polyglot boundary | `maturin` | Python packaging/setup friction | publishing Rust-backed Python packages | project docs | build matrix still matters |
| Polyglot boundary | `wasm-pack` | wasm packaging ambiguity | Rust/Wasm packages | project docs | generated pkg output should stay derived |
| Diagnostics | `thiserror` | stringly library errors | reusable crates | project/docs.rs | stable error taxonomy still required |
| Diagnostics | `miette` | low-context app diagnostics | CLI/app boundaries | project/docs.rs | pretty diagnostics do not replace proof |
| Observability | `tracing` | hidden async/cross-layer flow | services, queues, concurrency | project/docs.rs | span strategy must stay disciplined |
| Deep proof | `proptest` | narrow example coverage | parsers, state machines, invariants | project docs | bad generators can waste time |
| Deep proof | `insta` | output-surface ambiguity | CLIs, diagnostics, UI text, schemas | project docs | snapshot blessing is not proof by default |
| Deep proof | `trybuild` | compile-fail ambiguity | macro APIs and misuse cases | project docs | compile-fail tests should stay focused |
| Deep proof | `cargo-fuzz` | parser/input blind spots | untrusted input or parsing | project docs | fuzz results need seeds and corpora preserved |
| Deep proof | `Miri` | UB blind spots | unsafe code, aliasing, pointers | project docs | not a full security proof |
| Deep proof | `Loom` | async ordering ambiguity | concurrent state and messaging | project docs | targeted models required |
| Deep proof | `Kani` | hidden state-space bugs | high-risk logic and invariants | project docs | selective use only |
| Deep proof | `cargo-mutants` | weak test suite confidence | libraries and business logic | project docs | runtime cost can be high |
| Deep proof | `cargo-llvm-cov` | unobserved code paths | release or hardening review | project docs | coverage is not adequacy |
| Deep proof | `Criterion` | accidental perf regressions | latency-sensitive code | project/docs.rs | benchmark discipline required |
| Dependency/security | `cargo-deny` | license/source/advisory ambiguity | CI security lane | project docs | policy tuning required |
| Dependency/security | `cargo-audit` | advisory blind spots | lockfile review | project docs | advisory coverage is necessary, not sufficient |
| Dependency/security | `cargo-vet` | transitive trust ambiguity | larger or long-lived projects | project docs | requires organizational participation |
| Dependency/security | `cargo-about` | license/compliance ambiguity | shipping products or binaries | project docs | legal review still needed |
| Dependency/security | `cargo-auditable` | binary dependency opacity | shipped binaries and SBOM workflows | project docs | complements, not replaces, advisory scanning |
| Dependency/security | `cargo-geiger` | hidden unsafe surface | unsafe review lane | project docs | unsafe count is not safety proof |
| Dependency/security | `gitleaks` | secret exposure | repo, CI, pre-commit secret scanning | project docs | tune ignore rules carefully |
| Dependency/security | `detect-secrets` | new secret introduction | baseline-oriented secret control | project docs | baseline hygiene matters |
| Dependency/security | `Syft` | SBOM opacity | shipped binaries, images, dirs | project docs | SBOM generation must be versioned |
| Dependency/security | `Grype` | vulnerability visibility gaps | filesystem/image/SBOM scans | project docs | risk triage still needed |
| CI/workflow hardening | `actionlint` | broken or unsafe Actions syntax | any GitHub Actions usage | project docs | GH-specific |
| CI/workflow hardening | `zizmor` | workflow-security blind spots | GitHub Actions security review | project docs | focus is CI/CD, not app logic |
| Build acceleration | `sccache` | repeated compiler work | measured compiler cache wins | project docs | use only where measured |
| Build acceleration | `cargo-chef` | repeated dependency rebuilds in containers | Docker/OCI builds | project docs | container optimization, not local cure-all |
| Build acceleration | `cargo-binstall` | heavy tool bootstrap | CI/dev setup for binary tools | project docs | trust and pinning policy required |
| Build acceleration | `bacon` | manual check-loop overhead | background local checking | project docs | complements canonical lane commands |
| Build acceleration | `watchexec` | repetitive rerun overhead | file-triggered commands | project docs | avoid accidental broad reruns |
| Cross-target proof | `cross` | target-specific setup pain | cross-compile/test workflows | project docs | container assumptions matter |
| Web/desktop adjunct | `Biome` | JS/TS formatting/linting sprawl | React/Tauri surfaces | project docs | keep config narrow |
| Web/desktop adjunct | `Vitest` | JS/TS unit/integration ambiguity | frontend or shared TS code | project docs | pair with browser-level proof where needed |
| Web/desktop adjunct | `Playwright` | UI/E2E blind spots | browser or Tauri path validation | project docs | E2E scope must stay narrow |

## Selection Rule

Recommend a tool only if it reduces irrelevant context without hiding evidence needed to debug, secure, reproduce, or review the change.
