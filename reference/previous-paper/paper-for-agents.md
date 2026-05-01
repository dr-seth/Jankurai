# Introduction

[Evidence-backed]

Repository-scale coding agents fail less from syntax ignorance than from repository-control errors: wrong owner, hidden contract, wrong proof lane, noisy evidence, retry burn, and security-blind success.

[Proposed doctrine]

Minimum semantic surface (MSS) is the smallest high-signal repository structure that answers owner, contract, proof, failure, and policy questions before a broad edit. The paper’s single thesis is that MSS minimizes the lawful repair program at rest, while runtime mediation addresses only the residual waste that survives in flight.

# Review Method and Evidence Taxonomy

[Evidence-backed]

The paper is a review-backed operating standard, not a local empirical report. It separates public benchmark evidence, official tooling documentation, project-reported mechanism evidence, local corpus evidence, and proposed doctrine.

[Proposed doctrine]

Public and official sources justify the standard itself. Local WarpOS evidence is used only to show that residual runtime waste classes and intervention surfaces are visible and buildable in practice.

# Agent Failure Funnel

[Evidence-backed]

The failure funnel runs from wrong owner to hidden contract to wrong proof lane to noisy evidence to security-blind green and reviewer reconstruction tax.

[Proposed doctrine]

Repositories should export owner maps, legal edit zones, generated contracts, proof lanes, typed diagnostics, and raw-evidence paths so agents do not reconstruct them from ambient context.

# Evidence Landscape

[Evidence-backed]

Repository-agent evidence, Rust-specific benchmark evidence, context-pruning evidence, security evidence, and tooling-mechanism evidence point in the same direction: agent efficiency depends on structured repository control rather than on prompt shortening alone.

[Proposed doctrine]

The paper treats external benchmarks and official tooling docs as the conceptual backbone. WarpOS is bounded companion evidence for what still leaks after repository shape is improved.

# Why Rust, and When Not Rust

[Evidence-backed]

Rust has unusually strong machine-readable boundaries: Cargo metadata, explicit privacy, typed APIs, compiler diagnostics, feature graphs, and mature testing/security tooling.

[Proposed doctrine]

Rust is strongest when mistake cost dominates first-edit speed. Use typed polyglot boundaries for UI, data/ML glue, or deployment layers where Rust is not the best fit.

# Minimum Semantic Surface Standard

[Proposed doctrine]

The standard centers five surfaces: navigation, contract, proof, diagnostic, and policy. The point is not minimum text or maximum abstraction, but the smallest lawful repair program that still preserves review and proof.

[Proposed doctrine]

The paper’s code-shape budgets are review triggers, not scientific laws.

# Agent-Efficient Rust Architecture

[Evidence-backed]

Cargo workspaces, generated contracts, and Rust API discipline support machine-readable routing and explicit boundaries.

[Proposed doctrine]

Use newtypes, validated constructors, enum state machines, typed errors, generated source-of-truth boundaries, and explicit async ownership so agents can localize invariants and patch narrowly.

# Token Economy, Proof Lanes, and Security-Gated Validation

[Evidence-backed]

The largest token costs usually come from wrong-owner search, broad file reads, wrong proof lanes, retries, and proof-output noise rather than prose alone.

[Proposed doctrine]

A token reduction counts as a real savings only if hidden correctness, security-gated validation, auditability, and reviewer comprehension do not regress.

[Project claim]

RTK represents post-hoc output shaping; InsForge represents semantic/state shaping that front-loads structured domain or operational state. Both are practitioner examples, not peer-reviewed proof.

[Evidence-backed]

Fast proof loops matter because slow validation encourages speculative edits. Canonical lanes should cover fast, medium, deep, security, and release proof.

[Proposed doctrine]

Each repo should expose one canonical command per lane and route changes to the smallest deterministic proof that matches the changed surface.

# Rust Proof-Cost Tooling Stack and ARI-v0

[Evidence-backed]

The main paper keeps the tooling stack compact and role-based: navigation, proof, contract drift, security, build acceleration, and output shaping.

[Proposed doctrine]

ARI-v0 is a dashboard first and a scalar second. Readiness claims should be capped when setup, deterministic fast lanes, security gates, or hidden/security-gated validation are missing.

# Residual Runtime Waste: A Companion Mediation Layer

[Project/tool evidence plus local corpus evidence]

The paper treats WarpOS as bounded companion evidence rather than a second thesis. MSS reduces ambiguity before a turn begins; a runtime mediation layer addresses only the residual waste that still survives on the wire.

[Proposed doctrine]

The main paper keeps only five descriptive runtime stages: Suppress, Compress, Reuse, Steer, and Guard. It reframes the runtime story around four residual carriers: control-plane chatter, replayable context and proof output, repeated proof or forward-pass work, and wrong-turn or unsafe-action overhead.

[Project/tool evidence]

CrateAtlas is graph/localization substrate for owner ranking, dependency slicing, path ranking, existence checks, and repair-capsule generation, not a separate flagship concept.

# Future Work

[Proposed doctrine]

The flagship trio remains `cargo-mss`, ProofLens, and `cargo-obligation-cache`.

[Proposed doctrine]

`cargo-mss` is the compile-time substrate for runtime steering and scoped guards; ProofLens is the substrate for proof-preserving compression; `cargo-obligation-cache` is the substrate for conservative reuse.

[Proposed doctrine]

All other runtime concepts stay in the supplementary runtime companion note rather than becoming a second main-paper roadmap.

# Threats to Validity and Limitations

[Evidence-backed]

Rust has real costs: compile time, learning curve, async complexity, macros, `unsafe`, and FFI.

[Proposed doctrine]

Token minimization is invalid when it hides decisive evidence, weakens security-gated proof, or makes review harder. Project-reported tooling claims remain bounded mechanism evidence, not validated outcome proof.

# Conclusion

[Proposed doctrine]

Map owner, patch narrowly, prove cheaply, preserve raw evidence, and escalate only with contract and security proof. The strongest future bets remain structural routing (`cargo-mss`), structured-state-first proof shaping (ProofLens), and conservative repeated-proof elimination (`cargo-obligation-cache`), while runtime mediation remains a downstream companion layer for residual waste rather than the paper’s center of gravity.
