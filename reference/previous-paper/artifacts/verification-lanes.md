# Verification Lanes

The goal is not maximum testing on every edit. The goal is the smallest deterministic proof that matches the changed surface.

## Canonical Commands

These are the forms the paper recommends. Repositories should wrap them in a task runner such as `just fast`, `just medium`, `just deep`, and `just security`.

| Lane | Use When | Canonical Commands | Required Output |
| --- | --- | --- | --- |
| Fast | Any code edit | `cargo fmt --all --check`; `cargo check --workspace --all-targets`; `cargo clippy --workspace --all-targets -- -D warnings`; targeted `cargo nextest run -p <owner>` | First red/green signal and owner of failures |
| Medium | Public behavior, API, feature, schema, or integration changes | workspace `cargo nextest`; doctests; snapshots; compile-fail tests; feature checks | Contract-level confidence |
| Deep | Unsafe, concurrency, parser, state machine, public crate, or performance-sensitive changes | fuzzing; `cargo miri test`; Loom; Kani; `cargo mutants`; `cargo llvm-cov`; Criterion | High-risk proof and residual risk note |
| Security | Authz, secrets, external input, dependencies, unsafe, FFI, network, filesystem, or agent-tooling changes | `cargo deny check`; `cargo audit`; `cargo vet`; `cargo geiger`; `gitleaks`; `detect-secrets`; abuse-case tests; SBOM or vuln scan | Security-specific pass/fail |
| Release | Before merging broad or high-impact changes | full CI; E2E; migrations; semver/public API checks; performance budget; SBOM | Ship-readiness evidence |

## Routing Rule

- Agents should not infer incantations from CI YAML.
- Each lane should have one repo-local command alias.
- Broad validation is justified when public contracts, feature matrices, dependencies, unsafe code, or multiple owners are touched.
