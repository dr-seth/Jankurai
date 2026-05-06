<!-- jankurai-badge:start -->
[![Jankurai score: 100/100](agent/jankurai-badge.svg)](agent/jankurai-badge.json)
<!-- jankurai-badge:end -->

<p align="center">
  <img src="assets/jankurai_github_header_transparent.png" alt="Jankurai: agent-native repository control plane" width="100%">
</p>

# Jankurai

[![jankurai CI](https://github.com/jeppsontaylor/Jankurai/actions/workflows/jankurai.yml/badge.svg)](https://github.com/jeppsontaylor/Jankurai/actions/workflows/jankurai.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Jankurai is an anti-vibe coding standard and local audit CLI for auditable AI-assisted merge. Its public loop is simple: find vibe artifacts, prove the merge, repair the repo.

- Turns ownership maps, proof lanes, generated zones, security boundaries, rolling scores, merge witnesses, and repair queues into files agents and humans can both read.
- Checks 37 stable HLT rule families and maps 260 vibe-coding failure rows into auditable controls, including release readiness, bad CI/Git/tooling behavior, secret sprawl, generated drift, false-green tests, UX proof gaps, and missing evidence.
- Starts with read-only reports, then lets teams adopt guidance, CI, hooks, and ratchets only when they choose.
- Leaves receipts: JSON/Markdown reports, score history, proof artifacts, and command evidence under predictable paths.

Jankurai is not a model, hosted AI service, or "open source AI" system. It is repository infrastructure for making merge decisions reproducible. In Jankurai, "proof" means repository-local evidence receipts, not formal proof of full program semantics.

## Install

Prerequisites: `git` and a Rust toolchain with `cargo` on `PATH`.

```bash
git clone https://github.com/jeppsontaylor/Jankurai.git
cd Jankurai
cargo install --path crates/jankurai --locked
jankurai --version
```

For demos or CI logs, force rich terminal output:

```bash
export JANKURAI_COLOR=always
export JANKURAI_PROGRESS=always
```

## Try Safely In 5 Minutes

Run the first pass from the repository you want to inspect. These commands read source files and write only the report paths you name under `target/jankurai/`.

```bash
mkdir -p target/jankurai

jankurai adopt . \
  --profile auto \
  --mode observe \
  --out target/jankurai/adoption-plan.json \
  --md target/jankurai/adoption-plan.md

jankurai audit . \
  --mode advisory \
  --json target/jankurai/repo-score.json \
  --md target/jankurai/repo-score.md
```

Expected artifacts:

| Artifact | Purpose |
| --- | --- |
| `target/jankurai/adoption-plan.md` | Recommended next steps and adoption level. |
| `target/jankurai/repo-score.md` | Advisory score, findings, hard caps, and repair queue. |
| `target/jankurai/score-history.jsonl` | One score-history row per audit run. |

## Adopt Levels

| Level | Command | Writes | Use When |
| --- | --- | --- | --- |
| Observe | `jankurai adopt . --mode observe` | Named report files only. | You want an inventory before Jankurai changes tracked files. |
| Agents | `jankurai init . --level agents --dry-run` | Agent guidance after you apply with `--yes`. | You want Codex, Claude, Cursor, Copilot, or another agent to follow the same local rules. |
| Full | `jankurai init . --level full --dry-run` | Full scaffold after review and `--yes`. | You want owner maps, proof lanes, generated-zone policy, docs, contracts/db placeholders, CI, and hooks. |
| Ratchet | `jankurai ci install . --github --mode ratchet --baseline <file>` | CI gate. | The team has accepted a baseline and wants to block regression. |

Ratchet mode is impossible without an accepted baseline. Start in observe or advisory mode, commit `agent/repo-score.json` as the baseline when the team accepts it, then install ratchet CI with `--baseline`.

## Daily Loop

```bash
jankurai context-pack . --changed <path> --max-tokens 6000 --out target/jankurai/context-pack.json --md target/jankurai/context-pack.md
jankurai prove . --changed <path> --plan-out target/jankurai/proof-plan.json --plan-md target/jankurai/proof-plan.md
jankurai audit . --changed-fast --changed-from origin/main --json target/jankurai/audit-fast.json --md target/jankurai/audit-fast.md --timings-json target/jankurai/audit-timings.json
jankurai audit . --mode advisory --json target/jankurai/repo-score.json --md target/jankurai/repo-score.md
jankurai witness . --changed-from origin/main --baseline agent/repo-score.json --out target/jankurai/merge-witness.json --md target/jankurai/merge-witness.md
```

`--changed-fast` is an advisory inner-loop scan. It inventories changed files plus required control files, skips score-history writes, and must be followed by the full audit before merge or release.

Preview before tracked writes:

```bash
jankurai init . \
  --profile rust-ts-postgres \
  --level agents \
  --dry-run \
  --plan-json target/jankurai/init-agents.json
```

Apply only after reviewing the plan:

```bash
jankurai init . --profile rust-ts-postgres --level agents --yes
jankurai adapters verify .
```

Then open your coding agent from the same repo root and ask it to:

```text
Read AGENTS.md, follow the jankurai standard, then run the proof lane for my change.
```

## Upgrade Jankurai

Audits check for available Jankurai upgrades automatically. The check is advisory only: audit never auto-applies an upgrade, never changes score reports with live version data, and silently continues when the network is unavailable.

When audit reports an available upgrade, run:

```bash
jankurai upgrade
```

For advanced review-only checks, preview what would change:

```bash
jankurai update . \
  --check \
  --out target/jankurai/update/update-plan.json \
  --md target/jankurai/update/update-plan.md
```

Set `JANKURAI_NO_UPDATE_CHECK=1` to disable audit-time upgrade checks. Use `jankurai update . --offline` in environments where explicit update checks must avoid network-backed version lookups.

## How Jankurai Handles AI-Agent Risk

Jankurai treats agent behavior as repository policy, not chat convention.

| Risk | Jankurai Control |
| --- | --- |
| Broad or surprising writes | `agent/owner-map.json`, generated-zone manifests, dry-run plans, and explicit apply flags. |
| Weak proof | `agent/test-map.json`, proof lanes, `jankurai lane`, `jankurai prove`, and receipt paths under `target/jankurai/`. |
| Prompt injection | Root `AGENTS.md`, thin provider adapters, and rules that keep untrusted context from changing trusted policy or tool permissions. |
| Generated drift | `agent/generated-zones.toml` identifies generated/read-only outputs and the source command that owns them. |
| Security regressions | Security policy artifacts, `jankurai security run`, dependency/secret checks, and private reporting guidance. |
| Lost context | JSON/Markdown reports, score history, repair queues, and final handoff receipts. |

The project does not send repository contents to a hosted Jankurai service. The CLI inspects local files and writes local artifacts. Any external tools you run through your coding agent remain governed by that agent and your environment.

## GitHub Action

Run Jankurai in GitHub Actions with the Marketplace action tag:

```yaml
name: Jankurai Audit

on:
  pull_request:
  push:
    branches: [main]

permissions:
  contents: read

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      - uses: jeppsontaylor/Jankurai@v0.8.0
        with:
          mode: advisory
      - uses: actions/upload-artifact@v7
        with:
          name: jankurai-audit
          path: |
            agent/repo-score.json
            agent/repo-score.md
            target/jankurai/jankurai.sarif
            target/jankurai/summary.md
            target/jankurai/repair-queue.jsonl
```

Inputs:

| Input | Default | Values | Purpose |
| --- | --- | --- | --- |
| `mode` | `advisory` | `observe`, `advisory`, `ratchet` | Selects audit strictness. |
| `baseline` | `agent/repo-score.json` | Any repository-relative JSON path | Baseline score file used by `ratchet` mode. |

The action emits `agent/repo-score.json`, `agent/repo-score.md`,
`target/jankurai/jankurai.sarif`, `target/jankurai/summary.md`, and
`target/jankurai/repair-queue.jsonl`. No secrets are required. The CLI installs
from the action checkout and runs locally on the GitHub-hosted runner.

## Control-Plane Surfaces

Jankurai works as a local control plane over a few repeatable surfaces:

| Surface | Commands |
| --- | --- |
| Adoption and drift | `adopt`, `init`, `update`, `doctor` |
| Bounded agent context | `context-pack`, `adapters verify`, `adapters sync`, `agent verify`, `hooks install` |
| Proof and evidence | `lane`, `proof`, `prove`, `proof-verify` |
| Audit and routing | `audit`, `witness`, `score diff`, `score trend`, `rules verify`, `issues export`, score history, repair queues |
| Security and UX evidence | `security run`, `ux ...` |
| Repair and expiry | `repair-plan`, `repair`, `optimize`, `waivers expire` |
| Reusable/public evidence | `registry`, `cell`, `bench`, `certify`, `govern`, `publish` |

The loop is intentionally ordinary: changed paths map to owners and proof lanes, commands leave receipts, audit turns evidence into findings, and repair plans keep follow-up bounded.

## Toolkit

Jankurai ships as a Rust workspace of focused crates. Install the core CLI with `cargo install --path crates/jankurai --locked`; companion crates are available as library dependencies or standalone binaries.

### Core Crates

| Crate | Purpose |
| --- | --- |
| [`jankurai`](crates/jankurai) | Audit CLI and standard enforcement engine. Scores repositories, generates findings, routes proof obligations, and writes JSON/Markdown evidence. |
| [`jankurai-proofbind`](crates/jankurai-proofbind) | Semantic surface routing and proof obligation binding. Maps changed paths to owners, proof lanes, and generated-zone policies. |
| [`jankurai-proofmark`](crates/jankurai-proofmark) | Changed-behavior proof receipt engine. Validates that proof plans produce runnable commands and writes audit-ready receipts. |

### Companion Tools

#### Tuiwright — Playwright-Style TUI Testing

[Tuiwright](docs/tuiwright.md) is a Rust-native, black-box testing framework for terminal user interfaces. It spawns real TUI applications in a real pseudo-terminal, drives keyboard/mouse/paste/resize input, maintains an accurate virtual terminal model, and provides Playwright-grade ergonomics.

| Crate | Purpose |
| --- | --- |
| [`tuiwright`](crates/tuiwright) | Core library: PTY driver, vt100 screen model, locators, auto-waiting assertions, PNG screenshot renderer, GIF recorder, JSONL trace writer. |
| [`tuiwright-cli`](crates/tuiwright-cli) | CLI binary for headless `tuiwright screenshot` and `tuiwright record` commands. |
| [`tuiwright-demo`](examples/tuiwright-demo) | Minimal crossterm counter app used as the integration test target. |

```rust
use tuiwright::{Key, Page, SpawnConfig};
use std::time::Duration;

let page = Page::spawn(SpawnConfig::new("my-tui").size(80, 24))?;
page.wait_for_text("Ready", Duration::from_secs(5))?;
page.press(Key::Enter)?;
page.screenshot("target/tuiwright/home.png")?;
```

Run the Tuiwright test suite:

```bash
just tuiwright-test
```

#### Bad-Behavior Reference Docs

The `docs/` directory includes anti-pattern catalogs covering common vibe-coding failure modes. These are curated from real agent sessions and referenced by Jankurai's audit rules:

| Doc | Scope |
| --- | --- |
| [BAD_RUST.md](docs/BAD_RUST.md) | Rust anti-patterns: unsafe misuse, error swallowing, mega-functions, trait misuse |
| [BAD_SQL.md](docs/BAD_SQL.md) | SQL anti-patterns: destructive migrations, missing rollbacks, lock contention |
| [BAD_PYTHON.md](docs/BAD_PYTHON.md) | Python anti-patterns: scope creep, product truth leaks, missing typed contracts |
| [BAD_CI.md](docs/BAD_CI.md) | CI anti-patterns: flaky tests, no gates, artifact gaps |
| [BAD_GIT.md](docs/BAD_GIT.md) | Git anti-patterns: force push, broad commits, missing context |
| [BAD_DOCKER.md](docs/BAD_DOCKER.md) | Docker anti-patterns: root execution, unbounded layers, missing health checks |
| [BAD_TYPE.md](docs/BAD_TYPE.md) | Type system anti-patterns: handwritten DTOs, missing generated clients |
| [BAD_release.md](docs/BAD_release.md) | Release anti-patterns: mutable tags/assets, skipped proof, missing provenance, no rollback |

### Registered Tools

Jankurai's tool adoption catalog ([`agent/tool-adoption.toml`](agent/tool-adoption.toml)) tracks which tools are active and their enforcement mode. Each tool produces evidence that feeds the audit loop:

| Tool ID | Mode | Purpose |
| --- | --- | --- |
| `audit-ci` | auto | CI audit integration and score gating |
| `proof-routing` | auto | Changed-path proof obligation routing |
| `proofbind` | advisory | Semantic surface binding validation |
| `proofmark-rust` | advisory | Rust-specific proof receipt engine |
| `security` | auto | Dependency, secret, and provenance scanning |
| `ux-qa` | auto | Playwright UX evidence and accessibility |
| `db-migration-analyze` | auto | Migration safety analysis |
| `contract-drift` | auto | Generated contract drift detection |
| `rust-witness` | auto | Rust build witness graph |
| `vibe-coverage` | auto | Vibe-coding coverage analysis |
| `tui-testing` | advisory | TUI black-box testing via Tuiwright |
| `release-bad-behavior` | advisory | Release tag, artifact, provenance, and rollback bad-behavior checks |

## Project Status

Jankurai is early but usable as a local Rust CLI and standard workspace. The current source tree includes audit, init, update, proof, repair planning, migration analysis, security evidence, UX QA, TUI testing, publication evidence, and the paper source for *Jankurai: Merge Witnesses for Evidence-Carrying AI-Assisted Pull Requests*.

Paper framing:

- The standard is stack-neutral.
- The CLI is a reference implementation.
- The Rust/TypeScript/React/Vite/PostgreSQL/exception-only-Python profile is non-normative.
- This workspace is Rust-first: agents must not add Python for repo tools, proof lanes, product truth, product services, authorization, direct PostgreSQL writes, or general backend glue. Python is allowed only for rare advanced ML/data library work with a dated exception under `python/ai-service`.
- Full audit remains the merge and release gate.
- Score is posture; the merge witness is the decision; conformance is pass/fail.

Compatibility posture:

- Public report schemas should remain compatible or receive explicit migration notes.
- Ratchet enforcement should be opt-in and baseline-backed.
- New agent-facing guidance should be deterministic, local, and reviewable.

Known open-source gaps:

- deeper conformance runner with observed per-fixture witness decisions
- accessible HTML or tagged PDF edition
- durable JEP/RFC governance docs and independent implementation path
- public evidence registry, badge policy, and release checklist

## Docs

- [Install guide](docs/install.md)
- [Adoption guide](docs/adoption.md)
- [Agent-native standard](docs/agent-native-standard.md)
- [Architecture](docs/architecture.md)
- [Testing and proof lanes](docs/testing.md)
- [Tuiwright TUI testing](docs/tuiwright.md)
- [Merge witness](docs/merge-witness.md)
- [Rolling score](docs/rolling-score.md)
- [Security tool matrix](docs/security-tool-matrix.md)
- [Audit rubric](docs/audit-rubric.md)
- [Language bad-behavior catalogs](docs/language-bad-behavior.md)
- [Release bad-behavior catalog](docs/BAD_release.md)
- [Migration engine](docs/migration-engine.md)
- [Mission](docs/mission.md)

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request. The short version:

```bash
cargo fmt --all
cargo test -p jankurai
just fast
just score
git diff --check
```

Keep `reference/` read-only, do not hand-edit generated artifacts, and route changed paths through `agent/owner-map.json` and `agent/test-map.json`.

## Security

Do not open public issues for suspected vulnerabilities. Use GitHub private vulnerability reporting for this repository:

https://github.com/jeppsontaylor/Jankurai/security/advisories/new

See [SECURITY.md](SECURITY.md) for supported versions, evidence lanes, and advisory handling.

## Support

Use [GitHub issues](https://github.com/jeppsontaylor/Jankurai/issues) for reproducible bugs, documentation gaps, and feature proposals. See [SUPPORT.md](SUPPORT.md) for what to include.

## License

Jankurai is licensed under the [MIT License](LICENSE).

## Citation And Paper

This repository is the working source for the paper *Jankurai: Merge Witnesses for Evidence-Carrying AI-Assisted Pull Requests*.

Current release: standard `0.8.0`, schema `1.5.0`, paper edition `2026.05-ed8`.

Public thesis line: *Find the vibe. Prove the merge. Repair the repo.*

- Paper PDF: [paper/jankurai.pdf](paper/jankurai.pdf)
- Paper source: [paper/jankurai.tex](paper/jankurai.tex)
- Agent-readable companion: [paper/jankurai.md](paper/jankurai.md)
- Mission: [docs/mission.md](docs/mission.md)
