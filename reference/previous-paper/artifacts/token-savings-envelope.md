# Token-Savings Envelope (Illustrative)

Status: supplement artifact for `main.tex` §7.4 (Illustrative Savings Envelope). This file carries the detailed decomposition that the 10-page manuscript references but does not have room to display. **Everything below is doctrine, not measurement.** Ranges are structured hypotheses against which `SecureETTS` measurements should later be judged; none of them are empirical proof that MSS improves agent outcomes on a public benchmark.

## Four-tier envelope

| Tier | Reduction band | Midpoint | What it includes |
|---|---|---|---|
| **0 — Baseline** | **0%** | 0% | unstructured repository, no best practices |
| **1 — Known best practices** | **45–55%** | ~50% | short `AGENTS.md` routers, owner maps, JSON diagnostics, proof-lane routing, `cargo-nextest` failure-first output, safe command-output filtering |
| **2 — + Reference implementation** | **60–70%** | **~65%** | tier 1 plus a shipping implementation of repository routing (`cargo-mss`-class manifest) and proof-output shaping (ProofLens-class contract) |
| **3 — + Flagships and upstream horizon** | **75–85%** | **~80%** | tier 2 plus `cargo-obligation-cache` for repeated-proof elimination, plus the three upstream-horizon ideas (`cargo-agentmode`, `#[agent(...)]`, `cargo-trace-autopilot`) |

Tiers compound multiplicatively over overlapping token pools; the midpoint of tier 3 is *not* the linear sum of individual per-system contributions.

## Per-tier decomposition by token class

Token classes are the eight rows of `main.tex` Table "Token classes and preferred controls" (instruction / navigation / file-read / reasoning / patch / proof-output / wrong-turn-recovery / review-transfer).

### Tier 1 — Known best practices (45–55%)

Aggregated from four published evidence families:

- **AGENTS.md evidence** — short routers help; broad instruction files can *hurt* inference cost by more than 20% [agentsMdEval2026, onImpactAgents2026].
- **Task-aware context pruning** — 23–54% token reduction with minimal performance impact [swePruner2026].
- **Repository-sufficient context compression** — 51.8–71.3% budget reduction with 5.0–9.2% better resolution [sweEffi2025].
- **Project-reported command-output filtering** — RTK reports 60–90% reductions on supported commands, treated as project-reported mechanism evidence [rtk2026, rtkArchitecture2026].

Applied layer-by-layer across instruction / navigation / proof-output token classes, these converge on roughly 45–55% reduction. The midpoint (~50%) is the mark used in Figure 2 of the main paper.

### Tier 2 — + Reference implementation (60–70%)

Adds two structural moves on top of best practices:

- **Repository routing** (`cargo-mss`-class manifest) — a compiled owner map, test map, contract map, feature matrix, unsafe ledger, and generated-zone declaration. Attacks navigation and file-read token classes. Under the assumption that owner-localization is 25–45% of total tokens on navigation-heavy tasks (ContextBench-supported range), the manifest captures most of that phase; marginal additional reduction over tier 1 is approximately **10–15 percentage points**.
- **Proof-output shaping** (ProofLens-class packet contract) — decisive-fact-preserving compression wrapping compiler, test, security, and deep-lane output. Attacks proof-output and wrong-turn token classes. Under the assumption that proof output is 40–50% of total tokens on iterated tasks (SWE-Effi-supported range), a contract that preserves decisive-fact recall ≥ 0.98 while compressing passing runs by 70–95% and failing runs by 40–80% captures most of that surface; marginal additional reduction over tier 1 is approximately **5–10 percentage points**.

Layered multiplicatively over tier 1, these produce a tier-2 total of roughly 60–70% (midpoint ~65%).

### Tier 3 — + Flagships and upstream horizon (75–85%)

Adds three more systems:

- **`cargo-obligation-cache`** — content-addressed invalidation of proof obligations. Attacks repeated-proof-output and retry token classes on iterated work. Under the assumption that ~30% of proof-output tokens go to re-proving obligations that remain valid (multi-patch task regime), a 40–70% skip rate with zero false greens yields approximately **5–8 percentage points** marginal reduction over tier 2. Requires tier 2 (manifests + ProofLens packets) as input.
- **`cargo-agentmode`** (upstream horizon) — native `rustc --error-format=agent-json` and `cargo <verb> --output=agent-json`. Obviates wrapper-layer overhead and eliminates the "wrapper disagrees with compiler" staleness class. If adopted upstream, amortizes across the entire Rust ecosystem; marginal reduction over tier 2 is approximately **3–6 percentage points** direct plus unquantified protective (the wrapper-staleness class disappears). Contingent on Rust RFC adoption.
- **`#[agent(...)]` compiler-verified metadata** (upstream horizon) — proc-macro attribute attaching owner/proof-lane/contract-id/risk/invariant to source items, validated at compile time, exported via rustdoc JSON. Eliminates manifest-vs-source drift class. Marginal reduction over tier 2 is approximately **2–4 percentage points** direct plus protective (drift becomes a compile error).
- **`cargo-trace-autopilot`** (upstream horizon) — trace-driven semantic-surface refactoring suggestions. 0% at t=0; 5–10% compounding over ≥100 tasks. Value depends on acceptance rate and task cadence.

Layered multiplicatively over tier 2 (60–70%) with the above marginal contributions, the tier-3 total lands at approximately 75–85% (midpoint ~80%). The high end of the range requires all three upstream-horizon systems to ship and their assumptions to hold; the low end assumes `cargo-obligation-cache` alone lands and the upstream ideas remain aspirational.

## Why these numbers are presented as ranges, not point estimates

V6 reviewer consensus (8/8) explicitly flagged exact percentage claims for unmeasured future systems as an over-trust problem. The V7/V8 framing uses **widened bands** and **doctrine labeling** for every tier. The midpoints in Figure 2 of the main paper ("~50%", "~65%", "~80%") are *shorthand for the range*, not single-point claims. A future version of the paper that reports `arc-bench` or public-benchmark measurements should replace these ranges with measured confidence intervals and move from "doctrine" to "measurement" in the caption.

## What must not regress

Tier 2 and tier 3 savings are *validated savings* only if — simultaneously —

- hidden-pass rate does not regress,
- security-gated-pass rate does not regress,
- auditability (raw-evidence recoverability by path and SHA-256) is preserved,
- reviewer reconstruction time does not rise.

A headline token reduction paired with any of these regressions is an unsafe compression, not a savings. This constraint is enforced both by the Safe-Compression Rule (main paper §7) and by the per-system falsification criteria in the flagship-concept artifacts (`cargo-mss-concept.md`, `prooflens-concept.md`, `cargo-obligation-cache-concept.md`).

## Sensitivity: where the ranges would move

- **If `cargo-mss` manifests prove staleness-prone under agent workloads**, tier 2 drops toward 60%.
- **If `cargo-obligation-cache` produces any measured false greens**, it is disabled; tier 3 drops toward 70% and the midpoint falls to ~72%.
- **If `cargo-agentmode` lands upstream and is adopted broadly**, tier 3 high end rises toward 85%.
- **If evidence-family assumptions under-predict on large monorepos** (e.g., proof-output tokens are 55–65%, not 40–50%), tier 2 and tier 3 rise by 2–4 percentage points each.
- **If `cargo-trace-autopilot` achieves high acceptance rates in practice**, tier 3 compounds into 82–87% over 200+ task windows.

## Reference to the paper and evaluation harness

- Main paper hook: abstract and §7.4 (Illustrative Savings Envelope).
- Evaluation harness: `arc-bench` (local reference implementation; scenarios `repo-shape`, `witness-loop`, `exceptions`).
- Per-system falsification: `cargo-mss-concept.md`, `prooflens-concept.md`, `cargo-obligation-cache-concept.md`, `upstream-horizon-concepts.md`.
- Claim discipline: `citation-index.md`.
