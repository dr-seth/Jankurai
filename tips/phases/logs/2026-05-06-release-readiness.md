# 2026-05-06 Release Readiness

## Start Receipts

- `rtk jankurai update --client-start --quiet`: pass
- `rtk git status --short`: dirty tree with scoring/security/proof hardening, CI/badge/docs updates, user-owned prose scan work, and untracked baseline/prose smoke files.
- `rtk git diff --stat`: 42 tracked files changed, 1179 insertions, 440 deletions before baseline bootstrap.
- `rtk cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`: failed on 3 mechanical lints:
  - `crates/tuiwright/src/input.rs`: manual ASCII range check
  - `crates/tuiwright/src/render.rs`: `len() > 0`
  - `crates/jankurai-proofmark/src/coverage.rs`: manual `unwrap_or_default`

## Dirty Tree Classification

- Prose scan preservation: `crates/jankurai/src/audit/prose.rs`, `helpers.rs`, `scan.rs`, `audit_smoke.rs`, `docs/audit-rubric.md`.
- Audit enforcement and ratchet: `crates/jankurai/src/main.rs`, `model.rs`, `audit/mod.rs`, `audit/baseline.rs`, enforcement and baseline smoke tests, `schemas/repo-score.schema.json`.
- Security and proof evidence: `commands/security.rs`, `audit/security_artifact.rs`, proofbind/proofmark crates and tests, `schemas/security-evidence.schema.json`, `agent/security-policy.toml`, `agent/proof-lanes.toml`.
- CI, badge, docs: `.github/workflows/jankurai.yml`, `action.yml`, `commands/ci.rs`, `commands/badge.rs`, `agent/badge.toml`, `README.md`, `docs/testing.md`, `docs/artifact-contracts.md`, `Justfile`.
- Clippy cleanup: `crates/tuiwright/src/input.rs`, `crates/tuiwright/src/render.rs`, `crates/jankurai-proofmark/src/coverage.rs`.

## Tip Reconciliation

- Evidence-terminal scoring: CI now runs quality, proof, strict security, UX, and fixture evidence before final audit.
- No candidate self-baseline: CI resolves `agent/baselines/main.repo-score.json` from protected main or committed baseline.
- Fail-closed ratchet: baseline parsing and ratchet mode are covered by new audit enforcement tests.
- Strict security/proof lanes: CI uses `--strict --profile ci` and proof lanes use `--mode required`.
- CI hardening: workflow has explicit permissions, concurrency, timeouts, SHA-pinned official actions, and SARIF upload.
- Badge integrity: badge config points at the accepted baseline, not ignored local score output.
- Archive hygiene: `._*` is ignored and final cleanup will scan for sidecars before handoff.

## Implementation Receipts

- Clippy lint fixes applied to the three reported mechanical issues. Full clippy rerun pending.
