# Future Concept Ranking

Status: V8 supplement artifact.

This table explains why the flagship trio in the main paper is `cargo-mss`, ProofLens, and `cargo-obligation-cache`.

## Concept-family consolidation

| Family | V7 names | Consolidated paper role |
| --- | --- | --- |
| Negative context / repair cone / irrelevance proof | `cargo-nullspace`, `cargo-hypercut`, `cargo-counterfactual`, `cargo-antictx`, `cargo-negspace` | `cargo-mss` core |
| Capsule / shadow workspace / projected micro-repo | `cargo-capsule`, `cargo-negscope` | `cargo-mss` materialization mode |
| Minimal distinguishing context / active probes | `cargo-needle` | `cargo-mss` disambiguation mode |

| Concept | Token-savings ceiling | Mistake-reduction potential | Buildability | Reviewability | Ecosystem dependency | Main-paper role |
| --- | --- | --- | --- | --- | --- | --- |
| `cargo-mss` | very high | very high | high | high | medium | flagship |
| ProofLens | high | high | high | high | low | flagship |
| `cargo-obligation-cache` | very high | medium | medium | medium | medium | flagship |
| AgentDiagnostic | medium | high | high | high | low | secondary |
| native `rustc` / Cargo agent mode | very high | high | low | medium | very high | horizon |
| compiler-verified `#[agent(...)]` metadata | high | high | low | high | very high | horizon |

## Why the flagship trio wins

- `cargo-mss` removes the largest structural waste early by compiling negative context: repair capsules, ignore certificates, widening leases, optional shadow workspaces, and active distinguishing probes.
- ProofLens attacks the largest visible proof-output waste while preserving decisive evidence and auditability, and should prefer structured-state packets over raw-log summaries whenever systems can expose them.
- `cargo-obligation-cache` attacks repeated proof work, which remains a large token sink after routing and compression have already improved.

## Why AgentDiagnostic moved out of the flagship trio

AgentDiagnostic remains important, especially for recovery and reviewer handoff, but its primary upside is mistake reduction and reconstruction quality rather than the largest token-savings ceiling.

## Practitioner note: RTK vs. InsForge

- RTK represents **output shaping**: reduce visible tokens after commands run, while preserving raw recovery paths.
- InsForge represents **semantic/state shaping**: front-load structured domain and operational state so the agent performs fewer discovery queries and less raw-log parsing.

Both are useful patterns for the paper, but both remain project-reported practitioner evidence rather than peer-reviewed proof.
