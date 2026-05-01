# Upstream Horizon Concepts

Status: supplement artifact for the V7 review-backed standard. These three ideas are the most ambitious in the project. The main paper mentions them in one sentence (Section 10, "Additional Systems and Upstream Horizon") because each depends on changes outside any single repository — upstream Rust RFC adoption, compiler-level proc-macro work, or cross-task agent telemetry. This file preserves the detailed thinking so the ideas are not lost.

None of the three is a repo-local drop-in. All three are framed as research proposals with falsification criteria, not as a roadmap.

## 1. Native Cargo and rustc Agent Output Mode

### Idea

Give `rustc` and `cargo` a first-class agent output mode that emits repair-packet-compatible JSON on every diagnostic and every proof-loop artifact. Proposal name: `cargo-agentmode` (the aspirational name for the upstream feature).

- `rustc --error-format=agent-json` — superset of today's `--message-format=json`, emitting the fields ProofLens packets require (primary span, error code, owning crate, suggestion, likely layer).
- `cargo <verb> --output=agent-json` — pass-through at the Cargo level for `check`, `test`, `build`, `clippy`, `nextest`, etc.

The format is a superset of today's `--message-format=json` and a subset of a ProofLens packet. Upstream adoption means every Rust project gets the shape without a wrapper, and wrapper-layer staleness (tool disagrees with compiler version) disappears.

### Phased proposal

1. **Phase 1** — publish an RFC on the agent-output JSON Schema, aligned with rustdoc JSON (RFC 2963). Reference implementation: ProofLens packet schema.
2. **Phase 2** — land a nightly `rustc` flag behind `-Z agent-output`.
3. **Phase 3** — extend `cargo` with pass-through flags on the common subcommands.
4. **Phase 4** — stabilize after feedback from agent platforms (Codex, Claude, MCP implementations).
5. **Phase 5** — wrapper tools (ProofLens, `cargo-witness`-class wrappers) obsolete their diagnose layer and become thin adapters over native output.

### Evaluation

Fraction of the Rust ecosystem emitting agent-shaped output natively; wrapper-overhead elimination (tokens saved by removing wrapper diagnose); round-trip accuracy on `arc-bench`'s witness-loop scenario.

### Falsification

The RFC stalls and wrapper tools continue to carry the format. Alternately, native output adds no measured token wins beyond the wrapper format (the wrapper was already close enough).

### Why this matters

This is the single highest-leverage future concept the project developed. A ProofLens wrapper can capture the format today; native upstream adoption amortizes the benefit across the entire Rust ecosystem without opt-in, and eliminates the "wrapper disagrees with compiler" staleness class that any out-of-tree wrapper fights forever. It is in the upstream horizon tier precisely because it is not a repo-local move; it requires Rust project RFC process, and the lead time is measured in years.

## 2. Compiler-Verified `#[agent(...)]` Source Metadata

### Idea

Attach MSS metadata directly to source items as a compiler-verified attribute, rather than letting it drift in `Cargo.toml` and `agent/*.toml` manifests.

```rust
#[agent(
    owner = "crates/domain",
    proof_lane = "medium",
    contract_id = "ENT-RECOVERY-001",
    risk = "high",
    invariant = "expired grace period cannot grant active recovery",
)]
pub enum EntitlementState {
    Active,
    Expired,
    Revoked,
}
```

A proc-macro attribute validates each field against the workspace's metadata contract at compile time, exports the attributes through rustdoc JSON so `cargo-mss` consumes them natively, and refuses to compile when (for example) a public item carries `public_api = true` without a `contract_id`.

### Build plan

1. **Phase 1** — proc-macro crate + `cargo-mss` resolver that reads rustdoc JSON attribute output.
2. **Phase 2** — workspace-level validation rules (every `public_api` item must have a `contract_id`; every `risk = "high"` item must have a security-lane trigger).
3. **Phase 3** — integrate with repair-bundle generation so patch receipts cite `contract_id` directly.
4. **Phase 4** — migration tool converting existing `agent/*.toml` manifests into attribute form.

### Evaluation

Fraction of public items carrying `#[agent(...)]`; drift rate (manifest-vs-source disagreements caught at compile time); reduction in auditor findings for owner-entropy and contract-duplication classes; compile-time overhead on large workspaces.

### Falsification

Compilation overhead is noticeable on large workspaces; attributes become boilerplate ignored by agents; the migration tool cannot handle cross-crate invariants without hand editing; the proc-macro approach cannot represent what TOML manifests represent today.

### Why this matters

MSS metadata today lives detached from the source it describes: the manifest in `Cargo.toml` says `crates/domain` owns `EntitlementState`, but a refactor that renames the type can leave the manifest stale. That is the drift class `cargo-aer` and `cargo-surface-lint` exist to catch. Moving the metadata into source (as verified attributes) turns drift into a compile error rather than a scan finding. It's a structural rather than heuristic fix.

This is upstream-horizon rather than repo-local because it wants integration with rustdoc JSON emission (to be consumed without recompiling every consumer's code), which is where the idea touches the toolchain.

## 3. `cargo-trace-autopilot`: Trace-Driven Semantic-Surface Refactoring

### Idea

All repo-local systems in the paper are *prescriptive*: they tell the agent what to do. `cargo-trace-autopilot` is *descriptive*: it reads the agent's own run traces (files opened, lanes run, tokens wasted, recovery loops, widen-lease requests, reviewer reconstruction time) and suggests structural repository improvements. Over many tasks, the repository becomes progressively more agent-friendly. This is the flywheel the other systems lack.

### Input trace format

An append-only JSON-Lines trace emitted by the repo-local systems (`cargo-mss`, ProofLens, `cargo-obligation-cache`, and PSCP-class wrappers). Each line is one event: file opened, lane invoked, lane verdict, proof-output tokens emitted, recovery loop entered, widen-lease requested, reviewer reconstruction time (where instrumented).

### Output

A scored list of structural suggestions keyed to specific files and crates:

- "Split `crates/application/src/orders.rs` (900 LOC, opened in 40% of tasks, edited in 8%)"
- "Promote `domain::recovery` to its own crate (reverse-dependency storm detected on 12 tasks)"
- "Add an `#[agent(owner=...)]` annotation to `ComplianceChecker` (owner disagreement on 12 tasks)"
- "Narrow `crates/utils` (junk-drawer signal: 4 unrelated concepts detected)"

Each suggestion carries an estimated token-saving projection under stated assumptions, an acceptance interface (human accepts / rejects / defers), and — when accepted — an AER record if the change would otherwise trip the auditor.

### Build plan

1. **Phase 1** — stable JSON Lines trace format emitted by the repo-local systems.
2. **Phase 2** — scoring engine consuming traces + `cargo-mss` manifests + auditor findings.
3. **Phase 3** — human-review interface (CLI + web view).
4. **Phase 4** — A/B measurement: compare accepted-suggestion branches against matched unaccepted windows.

### Evaluation

- Acceptance rate of suggestions.
- Token-savings delta after accepted suggestions land (vs. matched unaccepted windows).
- Long-horizon Agent Readiness Index trend.
- Auditor-finding trend.

### Falsification

Suggestions become noise reviewers ignore; accepted suggestions do not move measured SecureETTS; the scoring engine over-suggests splits that fragment state machines; long-horizon ARI does not improve.

### Why this matters

The other systems are static-point interventions: ship them, measure, done. `cargo-trace-autopilot` is the monotone-improving loop. It starts at zero savings (no traces yet, no accepted suggestions) and compounds as the corpus of accepted suggestions grows. Its high end depends on acceptance rate and task cadence, which is why it sits in the upstream-horizon tier alongside the RFC-dependent systems — it is a research loop, not a build-and-ship artifact.

## Why these three live in the supplement, not the main paper

Each one is high-value but **ecosystem-bound** rather than repo-local. The paper's main-body Future Work is "what can a team unilaterally ship to push the envelope." These three are "what the Rust ecosystem could ship to push the envelope further." They are the deepest long-term ideas the project developed, and they belong in the research-agenda tail of the paper family rather than in a 10-page IEEE manuscript that commits to three buildable flagships.

If any of the three move from aspiration to visible progress — an accepted RFC, a working proc-macro, a measured telemetry loop — the next version of the paper should promote them into the main Future Work section and open a new upstream-horizon appendix for whatever is next.
