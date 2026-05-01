# `AgentDiagnostic` and Proof-Carrying Patches

Status: proposed research interface, not implemented.

## Goal

Turn Rust failures into typed recovery instructions and require patches to carry proof receipts. The pair should tell an agent what failed, who owns it, which invariant matters, what edits are allowed, what edits are forbidden, which proof lane comes next, and what evidence the final patch preserves.

## Sketch

```rust
pub trait AgentDiagnostic {
    fn code(&self) -> &'static str;
    fn owner(&self) -> &'static str;
    fn invariant(&self) -> &'static str;
    fn likely_cause(&self) -> &'static str;
    fn safe_next_step(&self) -> &'static str;
    fn forbidden_edits(&self) -> &'static [&'static str];
    fn proof_lane(&self) -> &'static str;
}
```

## Integration Points

- `thiserror` for stable library error variants.
- `miette` for rich application diagnostics and source spans.
- `tracing` for span-linked runtime failures.
- nextest, rustc JSON, Clippy, cargo-deny, cargo-audit, fuzzing, Miri, and CI reports for repair packet generation.

## Repair Packet Fields

- Stable diagnostic code, owner, invariant, likely cause, safe next step, forbidden edits, proof lane, evidence paths, raw-output hash, and security flag.

## Proof-Carrying Patch Receipt

- Changed owners, contracts touched, generated files touched, unsafe/FFI changes, dependency changes, commands run, raw-output hashes, security-gated lane status, and residual risk.

## Evaluation

Compare raw failures, ordinary human-written failure messages, repair packets, and proof-carrying patches. Measure recovery tokens, wrong-turn tax, failure-to-owner time, next-action correctness, repeated-failure rate, hidden pass, security-gated pass, and human review burden.

## Falsification

The standard fails if repair packets add boilerplate without improving recovery, cause agents to over-trust stale advice, or make maintainers avoid precise diagnostics because the interface is too heavy.
