# `cargo-mss`: Negative-Context Semantic-Surface Compiler

Status: proposed research tool, not implemented. V8 flagship future system of the paper.

## Goal

Generate a compiled *task-conditioned semantic surface* that answers the agent's first questions before it reads broadly or edits: who owns this surface, what contracts matter, what files are legal to touch, what features apply, what proof lane is required, what widening policy applies, what evidence must be preserved, and what can be safely ignored until contradiction appears. `cargo-mss` turns doctrine (prose guidance in `AGENTS.md`, CODEOWNERS, unsafe ledgers) into machine-readable structure agents can query, rather than reconstruct each session from `ls`, `rg`, and broad file reads.

## Inputs

- `cargo metadata --no-deps --format-version 1` — package, target, feature, and workspace shape.
- rustdoc JSON (RFC 2963) or rust-analyzer symbol data — public API, modules, traits, impls, re-exports, visibility.
- `CODEOWNERS` — ownership mapping.
- `agent/generated-zones.toml` — generated-file markers, source-of-truth back-pointers, regeneration commands.
- `agent/unsafe-ledger.toml` — `unsafe` blocks, SAFETY justifications, invariants, required proof lanes.
- `agent/dependency-rationale.toml` — per-crate rationale, license policy, approval requirements.
- `proof-lanes.toml` — canonical lane commands and budgets.
- `contracts/` — source-of-truth schemas (OpenAPI, Protobuf, SQL, JSON Schema).
- Test inventory — Cargo targets, `cargo-nextest` lists, doctests, integration tests, snapshots, fuzz targets, security checks.
- Task anchors — failing test, compiler diagnostic, panic span, issue text, diff, security finding, schema diff.

## Outputs

| Artifact | Contents |
|---|---|
| `repair_capsule.json` | owner rank, legal edit zone, required proof lane, raw evidence handles, security triggers, widening predicates |
| `ignore_certificate.json` | hard-irrelevant and soft-irrelevant surfaces, exclusion reasons, freshness/hash, contradiction triggers |
| `edit_grammar.json` | admissible edit operators such as enum transition, validated constructor, trait-body update, generated-contract source update, error variant, or regression/property test |
| `.mss-shadow/` | optional projected micro-workspace with full source in-slice and read-only stubs out-of-slice |
| `agent-map.json` | owners per path and symbol, reverse-dependency index, risk tags, `api_surface_hash`, `proof_density` per crate |
| `test-map.json` | source-to-proof mapping: per crate, the owned tests, doctests, integration harnesses, reverse-dep tests, smoke tests, e2e gates, estimated cost, and selection reason |
| `contract-map.json` | public boundary shapes: Rust type → schema file → generated TS/Proto/SQL output; source-of-truth marker |
| `feature-matrix.json` | relevant feature combinations, skipped combinations with reasons, per-feature-set proof-lane hints |
| `generated-zones.json` | derived from `generated-zones.toml` + filesystem sweep; the authoritative "do not edit" list |
| `unsafe-ledger.json` | structured view of the unsafe ledger with required proof lanes |
| `dependency-rationale.json` | per-dep owner, allowed features, license/security notes, approval policy |
| `proof-lanes.toml` | same shape as input, but with `cargo-mss`-resolved placeholders (e.g., `${owner}`) expanded per ARC |
| `widening-policy.toml` | when an agent may cross crate, public-API, generated-code, dependency, or unsafe boundaries; widen-lease triggers |
| `staleness.json` | hash of inputs; used by CI to gate against drift |

## Query surface (CLI and MCP-exposable)

```
cargo mss map                               # emit all manifests
cargo mss owner <path|symbol>               # who owns this
cargo mss proof <issue|path>                # required proof lane(s)
cargo mss contract <symbol>                 # boundary shape + source-of-truth
cargo mss security <path>                   # does this path trigger security gate?
cargo mss diff <git-range>                  # what changed; minimum sufficient proof
cargo mss locate "<natural-language issue>" # ranked candidate owners
cargo mss validate                          # fail if manifests are stale
```

The `locate` subcommand is the issue-to-context compiler. It folds the earlier negative-context, capsule, and needle ideas into one interface: given an issue description, emit the minimum sufficient set of files, public signatures, nearby tests, invariants, feature flags, forbidden zones, distinguishing probes, and the first proof command.

## Build plan (phased)

1. **Phase 1** — ingest Cargo metadata and rustdoc/rust-analyzer semantics; emit `agent-map.json`, `contract-map.json`, and a first repair hypergraph.
2. **Phase 2** — join CODEOWNERS, generated-zone markers, tests, and proof lanes; emit `test-map.json` and `widening-policy.toml`.
3. **Phase 3** — accept task anchors and compute the obligation cone by backward-cause slicing, forward proof slicing, contract/risk expansion, and legal-edit filtering.
4. **Phase 4** — emit `repair_capsule.json`, `ignore_certificate.json`, and `edit_grammar.json`.
5. **Phase 5** — add active distinguishing probes and optional `.mss-shadow/` materialization.
6. **Phase 6** — CLI + MCP adapter, staleness hashing, and CI gating that fails when maps or certificates are out of date.

## Integration points

- Consumed by the paper's other two flagship systems: `ProofLens` reads `contract-map.json` and `unsafe-ledger.json` to produce Rust-specific failure packets; `cargo-obligation-cache` reads `agent-map.json`, `contract-map.json`, and `feature-matrix.json` to compute per-obligation invalidation keys.
- Consumed by agent platforms (Codex, Claude, MCP) as a read-only surface.
- Emits traces used by `cargo-trace-autopilot` (upstream horizon; see `upstream-horizon-concepts.md`).

## Evaluation design

Compare agents with and without `cargo-mss` on the same Rust repair tasks. Primary metrics: token-to-owner, files-opened-before-owner, wrong-owner edit rate, illegal-touch rate, wrong-proof-lane rate, hidden-pass rate, security-gated-pass rate, reviewer reconstruction cost, and `SecureETTS`. Secondary metrics: manifest generation time, staleness rate, widen-lease rate, shadow-workspace escape rate, and reviewer trust in generated outputs.

## Failure modes

- **Staleness.** Generated maps go out of date faster than builds regenerate them; agents trust them; illegal touches rise. Mitigation: CI-gate `cargo mss validate` and require a fresh `staleness.json` before every `plan`.
- **Over-trust.** Agent edits exactly what the repair capsule says is legal even when it is wrong. Mitigation: `widening-policy.toml` with widen-lease triggers forces escalation on risky cases.
- **Shadow mismatch.** The projected workspace hides a dependency or contract that the full repo still needs. Mitigation: shadow mode is optional and conservative; contradictions force widening rather than blind reuse.
- **Manifest bloat.** Manifests grow larger than the files they replace. Mitigation: size budgets per artifact; `cargo mss validate` fails on bloat.
- **Maintenance cost.** Upkeep of the input files (ledgers, zones, rationale) exceeds savings. Mitigation: per-crate adoption, not all-or-nothing.

## Falsification criterion

`cargo-mss` fails if, on a matched benchmark, agents using it show *no* measurable reduction in files-opened-before-owner, illegal-touch rate, reviewer reconstruction cost, or `SecureETTS` versus a baseline reading the same repo without the compiler. It also fails if the repair capsule produces false greens, if staleness makes widening constant, or if shadow projection hides a dependency the full proof still requires.

## Why this is the flagship repo-local concept

It attacks the single largest structural waste class in agent traces (wrong-owner search + broad file reads) and compounds with the other two flagships: `ProofLens` cannot produce Rust-specific conflict packets or structured-state routing without `contract-map.json` and `unsafe-ledger.json`, and `cargo-obligation-cache` cannot compute content-addressed invalidation without the compiler's hashes and repair cones. Everything downstream depends on this.
