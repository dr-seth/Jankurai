# ProofLens: Proof-Preserving Rust Output Compression

Status: proposed research tool, not implemented. Flagship future system of the paper.

## Goal

Compress Rust proof output without hiding the facts needed to debug, secure, reproduce, or review a change. ProofLens is the stricter research version of generic command-output token filtering: **fewer tokens only count when hidden pass, security-gated pass, auditability, and human-review quality are preserved.** RTK and similar project-reported filters demonstrate that raw proof output is highly compressible; practitioner systems such as InsForge show the complementary value of exposing structured operational state instead of forcing agents to reconstruct it from logs. ProofLens therefore adds a mandatory evidence contract so that compression is *auditable-safe* rather than informally convenient and prefers structured state packets whenever the substrate can expose them.

## Input commands wrapped

Compiler and type-checker:
- `cargo check --workspace --all-targets --message-format=json`
- `cargo clippy --workspace --all-targets --message-format=json -- -D warnings`

Test and property-based:
- `cargo nextest run` with JSON/libtest-json output
- `cargo test --doc`
- `cargo-mutants` and `cargo-llvm-cov` output

Deep-lane:
- `cargo miri test`
- `cargo fuzz run <target>`
- `cargo loom test` and `cargo kani verify`

Security and supply chain:
- `cargo-deny`, `cargo-audit`, `cargo-vet`, `cargo-geiger`, `cargo-semver-checks`
- `gitleaks`, `trufflehog`, `actionlint`

Observability:
- `tracing` span output
- snapshot diffs (`insta`) and schema diffs (`cargo-public-api`)
- structured operational state such as deployment/build state when available

## Required preserved fields (the packet contract)

Every packet MUST include the following. A packet missing any of these is *not* a valid ProofLens packet and raw escalation is required.

| field | why |
|---|---|
| `command` | reproducibility |
| `cwd` | reproducibility |
| `exit_code` | pass/fail signal cannot be compressed away |
| `tool_version` | matches the obligation-cache invalidation key |
| `lane` | fast / medium / deep / security / release |
| `owner_arc` | which crate / module owns the failure |
| primary error or failing-test identity | decisive fact |
| `panic_text` (when present) | decisive fact |
| advisory IDs (when present) | decisive fact |
| `fuzz_seed` (when applicable) | reproducibility |
| `primary_span` (file, line, column) | decisive fact |
| `raw_log_path` and `raw_log_sha256` | auditability route |
| `redaction_status` | security |
| `truncation_status` | compression honesty |

## Rust-specific packet types

Beyond the generic packet, ProofLens defines Rust-specific packet variants for the failure classes that dominate Rust repair:

- **borrow** — lifetime, `'static` bound, `&mut` aliasing
- **trait** — unsatisfied bound, orphan rule, object-safety, coherence
- **async** — `Send`/`Sync` at spawn boundary, `MutexGuard` held across `.await`, cancellation
- **cfg / feature** — additive-feature violation, feature-combination failure, target-specific `cfg`
- **proc-macro** — derive or procedural-macro expansion failure (paired with `cargo expand` reference)
- **unsafe / FFI** — SAFETY-comment missing, Miri UB, FFI layout/ABI mismatch
- **schema-diff** — generated artifact out of sync with source of truth
- **runtime-state** — structured deployment/build/runtime state packet emitted directly by the system rather than inferred from logs

Each Rust-specific packet type extends the generic contract with additional preserved fields — e.g., the `async` packet preserves the span boundary where `.await` is held, the `trait` packet preserves the full trait-resolution trace, the `unsafe` packet preserves the ledger reference.

## Outputs

- **Summary** optimized for agent consumption: `lane=fast status=red owner=crates/domain primary=E0277 at state.rs:88 raw=artifacts/raw/check-2026-04-23T10-42Z.log sha256=…`
- **`proof-receipt.json`** — the machine-readable packet.
- **`raw-log`** (tee'd, not model-visible) — full compiler / nextest / tool output, addressed by sha256.
- **Optional escalation hint** — when `redaction_status=applied` or `truncation_status=lossy`, the summary ends with "escalate: cat $raw_log_path" and the agent must fetch raw output rather than trust the summary.

## Structured-state-first rule

If the underlying system can expose structured execution state directly, ProofLens should serialize that state as a first-class packet rather than summarize logs that happen to imply it. In practice this creates three layers:

1. **Raw log compression** for tools that only emit text.
2. **Structured proof packets** for tools that already emit JSON or machine-readable diagnostics.
3. **Structured operational-state packets** for systems that can expose deployment/build/runtime state directly.

This distinction matters because structured state can eliminate whole classes of log-parsing tokens rather than merely compressing them after the fact.

## Evaluation design

Compare four conditions on the same Rust repair-task corpus:

1. Raw output (no compression).
2. Generic command-output filtering (RTK or similar).
3. Naive truncation (first 1000 lines + last 500).
4. Structured-state packets where available.
5. ProofLens with the full packet contract.

Primary metrics:

- **Compression ratio** (visible tokens saved, per command).
- **Decisive-fact recall** — fraction of decisive facts present in the summary vs. present in raw. Target: ≥ 0.98.
- **False-green rate** — agent accepts a green summary that hides a red signal. Target: 0.
- **Raw-escalation rate** — how often the agent legitimately falls back to `raw_log`. Healthy: low but non-zero; zero is suspicious (agent is over-trusting the summary).
- **Token-to-first-red** and **token-to-first-green** — the tokens between task start and the first decisive proof signal.
- **Hidden-pass rate** and **security-gated-pass rate** under each condition.
- **SecureETTS** under each condition.

Secondary metrics:

- Reviewer reconstruction time (human reviewer reads the receipts + raw logs and reproduces the failure chain).
- Size of the packet schema (we want small, stable shapes).
- Rust-specific packet-type coverage (fraction of real Rust failures that match one of the defined types).

## Integration points

- **Consumes** `cargo-mss` output: `contract-map.json` and `unsafe-ledger.json` drive owner and contract attribution in packets.
- **Feeds** `cargo-obligation-cache`: every ProofLens packet is also a candidate proof certificate for the obligation graph.
- **Feeds** reviewer tooling (patch receipts, AgentDiagnostic in the supplement).

## Failure modes

- **Hidden decisive fact.** Compression drops a field that matters (seed, advisory, span). Mitigation: the packet contract is enforced by schema validation; a packet missing a required field fails to serialize.
- **Packet-type drift.** Rust evolves new failure shapes (new error codes, new trait-resolution errors) and existing packet types miss them. Mitigation: the `generic` packet type always applies; Rust-specific types are additive.
- **Redaction over-aggressive.** Security redaction drops something the agent needs to fix. Mitigation: redaction is logged in the packet; raw escalation recovers the original.
- **Over-compression in passing runs.** Passing runs get near-empty packets, so any flip to failing is harder to debug. Mitigation: passing-run packets still preserve tool-version and duration; failing-run packets preserve the full contract.

## Falsification criterion

ProofLens fails if any of:

- Compressed summaries hide decisive compiler, test, panic, unsafe, dependency, or security evidence (any decisive-fact recall below 0.98).
- Agents pass visible checks while regressing hidden or security-gated validation more often than with raw logs.
- Reviewers cannot reconstruct the failure chain from packet + raw-log handle in reasonable time.
- The packet contract is so strict that the schema rejects a real tool output the paper claims to cover.

## Why this is the flagship proof-output concept

Proof-output tokens are the largest visible-token bucket after navigation on iterated tasks. RTK-class filtering already captures the easy case on passing runs. ProofLens is the version of the idea where compression is *safe by contract* rather than safe by convention — which is the line between a practitioner tool and a research contribution.
