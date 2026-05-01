# `cargo-obligation-cache`: Content-Addressed Proof-Obligation Engine

Status: V8 flagship future-work concept. Proposed research tool, not implemented.

## Goal

Reduce repeated proof cost without creating false greens. Standard file-level caching (build-script output, incremental compilation, `sccache`) is too coarse for agent verification loops: it caches artifacts but does not reason about whether a proof remains *valid*. `cargo-obligation-cache` caches **proof obligations**, content-addressed against exactly the inputs that determine whether a proof's result can be trusted. In the flagship trio, `cargo-mss` narrows the repair cone first, ProofLens narrows the visible evidence surface second, and `cargo-obligation-cache` removes repeated reruns once the right scope has already been found.

## Core idea

Every proof lane decomposes into a set of obligations. An obligation is a claim like *"the public API of crate `domain` has no semver break under the default feature set on `x86_64-unknown-linux-gnu` with rustc 1.93.0 and `cargo-semver-checks` 0.38,"* or *"the `entitlement_replay_rejects_expired_grace` test passes under feature `grace-periods` with nextest 0.9.x."* Each obligation has a *certificate* (a `proofcert`) recording command, cwd, exit code, tool versions, decisive facts, raw-log path/hash, and the set of inputs the obligation was computed over.

A diff invalidates the minimum set of obligations it can actually affect, not all of them. A pure-docs edit invalidates zero proof obligations. A public-API change in `domain` invalidates every obligation keyed on `domain`'s public API hash — but not obligations keyed only on a different crate's API. This is the safety-preserving counterpart to the interface-hash vs implementation-hash trick that `cargo-witness` already uses for test selection.

## Invalidation keys (the content-addressed tuple)

Each obligation is keyed on a tuple of hashes; any change to any element invalidates the obligation:

- **public API hash** — per-crate, SHA-256 over sorted public-item signatures (consumes `cargo-mss`'s `contract-map.json`).
- **contract hash** — per-boundary, SHA-256 over the source-of-truth schema file plus the generator command and version.
- **feature set hash** — sorted feature list for the build in which the obligation was checked.
- **target triple** — e.g., `x86_64-unknown-linux-gnu`.
- **toolchain and tool versions** — `rustc`, the specific proof tool (`cargo-nextest`, `cargo-miri`, `cargo-audit`, etc.).
- **unsafe-ledger state hash** — hash of `agent/unsafe-ledger.toml` (consumes `cargo-mss`).
- **dependency set hash** — sorted list of resolved dependencies from `Cargo.lock`.
- **relevant test and fixture hashes** — for test obligations, hashes of the test source and any fixtures it reads.

If any hash changes, the obligation is invalidated; the agent reruns the lane that covers it. If no hash changes, the obligation remains valid and the cache returns the prior certificate.

## Outputs

| Artifact | Contents |
|---|---|
| `proof-obligation-graph.json` | the DAG of obligations; each node has its invalidation-key tuple and the lane that produces it |
| `proof-certificates.json` | the certificate store; content-addressed; one entry per `(obligation, invalidation-key)` pair |
| `proof-invalidation.json` | per-diff analysis: which obligations are still valid, which are invalidated, and the specific reason (e.g., "`domain` public API hash changed: `EntitlementState::Revoked` added") |
| `proof-rerun-plan.json` | the minimal set of lanes to rerun, plus estimated token and wall-clock savings vs. a full rerun |

## Pipeline (phased build plan)

1. **Phase 1** — JSON Schema for the proofcert and the obligation-graph DAG. Integrate with `cargo-mss` output so the graph is pre-populated from `contract-map.json`, `feature-matrix.json`, `unsafe-ledger.json`, and `test-map.json`.
2. **Phase 2** — instrument `cargo-witness diagnose` (or its future replacement) and `cargo-nextest` to emit proofcerts as a by-product of every lane execution.
3. **Phase 3** — the diff-to-invalidation rule engine: given a git diff + the `cargo-mss` manifest, compute which hashes changed and therefore which obligations are invalidated.
4. **Phase 4** — shared CI store (S3-class) with content-addressed keys, so a PR branch can reuse obligations its base branch already proved.
5. **Phase 5** — stale-receipt detection (toolchain bump, dependency delta, feature regeneration) with conservative fallback rules: when in doubt, rerun.
6. **Phase 6** — obligation-graph visualization: reviewers can open a web view and see "why was the medium lane skipped?" with the specific still-valid obligations that justify the skip.

## Query surface

```
cargo obligation-cache plan <git-range>
cargo obligation-cache explain <obligation-id>
cargo obligation-cache stats                        # hit rate, token savings, stale rate
cargo obligation-cache invalidate <pattern>         # manual override (audited)
```

## Integration points

- **Consumes** `cargo-mss` manifests (public API hash, contract hash, feature matrix, unsafe-ledger state hash).
- **Consumes** ProofLens packets, which already carry most of the fields a proofcert needs.
- **Complements** build caches (`sccache`, `cargo-chef`) — those cache *artifacts*; this caches *obligations*. Same diff can be a cache hit for both without contradiction.

## Evaluation design

Measure:

- **Repeated-proof token reduction** on multi-patch tasks (10+ patches per task), split by lane.
- **Repeated-proof wall-clock reduction**.
- **Stale-receipt rate** — fraction of cache hits that would produce a different result if rerun. Target: 0.
- **False-green rate** from accepted skips. Target: 0.
- **Reviewer trust in skip explanations** (survey, ordinal 1–5; target ≥ 4).
- **Hidden-pass rate** and **security-gated-pass rate** with cache vs. without.
- **SecureETTS** on iterated tasks.
- **Obligation-graph coverage** — fraction of proof-output tokens covered by cache-aware skips.

Compare three conditions: no cache, file-level cache (`sccache` baseline), obligation cache.

## Failure modes

- **Stale-receipt false green.** Hash collision, or a hash key that fails to capture a relevant input (e.g., an environment variable that affects behavior but isn't hashed). *Any* false green is a correctness bug, not a tuning issue.
- **Conservative over-invalidation.** The rule engine invalidates more than it needs to; measured savings approach zero. Mitigation: track over-invalidation separately from false-green risk; tune keys without relaxing correctness.
- **Overhead exceeds savings on small workspaces.** Hash computation + store round-trips cost more than the saved proofs. Mitigation: opt-in; gate on workspace size or proof-lane cost threshold.
- **Reviewer distrust.** Reviewers ignore skips and rerun everything manually. Mitigation: the obligation-graph visualization is the authority; skips that cannot be explained are disabled.
- **Store poisoning.** A malicious or accidental cache entry claims a proof that was never run. Mitigation: certificates carry the raw-log SHA-256 and are revoked if the raw log cannot be produced.

## Falsification criterion

The concept fails if any of:

- Stale receipts create a false green on any task.
- Reviewer reconstruction becomes *harder* than rerunning the lane (measured by reconstruction time and survey trust).
- Invalidation overhead outweighs saved proof work on ordinary patch loops (net token delta ≤ 0).
- The obligation graph becomes so large it cannot be rendered or reasoned about; reviewers stop using it.

## Why this is the third flagship

Navigation waste (`cargo-mss`) and proof-output waste (ProofLens) are the two largest visible-token buckets in single-task traces. Repeated-proof waste is the *compounding* waste on iterated tasks — the cost that scales with number of patches, not tasks. A 30–60% skip rate on repeated proofs with zero false greens compounds with PSCP routing and is the single biggest remaining ceiling once navigation and proof-output are under control.
