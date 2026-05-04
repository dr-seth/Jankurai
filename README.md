<img src="assets/jankurai_header.png" alt="Jankurai" width="100%">

# Jankurai

Jankurai is a control plane for agent-native repositories. It makes agent guidance, ownership, proof lanes, generated zones, scoring, CI evidence, and repair queues machine-readable so generated code is easier to reject, route, and repair.

The adoption ladder is progressive:

```text
agents -> score -> ci -> full -> ratchet
```

Start with agent/provider hooks, add local scoring when ready, run CI in observe mode before enforcing anything, install the full scaffold only when the repository wants the whole control plane, and ratchet only after accepting a baseline.

## Install

While public packaging is being prepared, install from this source checkout:

```bash
cargo install --path crates/jankurai --locked
jankurai --version
```

## Minimal Agent Install

Preview the smallest install. It writes nothing during dry run and emits a machine-readable plan:

```bash
jankurai init . --level agents --dry-run --plan-json target/jankurai/init-agents.json
```

Apply only root/provider guidance:

```bash
jankurai init . --level agents --yes
jankurai adapters verify .
```

This level installs `AGENTS.md`, `agent/JANKURAI_STANDARD.md`, `agent/MASTER_PLAN.md`, and thin provider adapters for Codex-style, Cursor, Copilot, Claude, Gemini, and other agent surfaces present in the selected profile.

## Add Scoring

Install local scoring manifests without CI or full scaffold docs:

```bash
jankurai init . --level score --yes
jankurai audit . --mode advisory --json target/jankurai/repo-score.json --md target/jankurai/repo-score.md
```

Score level adds the owner map, test map, generated-zone manifest, proof lanes, audit policy, standard version, and minimal `Justfile` recipes. It is meant for ad hoc scoring, not conformance claims.

## Observe CI

Install observe-mode CI after local scoring is useful:

```bash
jankurai init . --level ci --yes
jankurai ci install . --github --mode observe --dry-run
jankurai ci install . --github --mode observe
```

Observe CI uploads score artifacts and repair queues but does not enforce score 85. Existing workflow files are left unchanged.

## Full Scaffold

Full remains the default for backward compatibility:

```bash
jankurai init . --profile rust-ts-postgres --dry-run --plan-json target/jankurai/init-full.json
jankurai init . --profile rust-ts-postgres --yes
```

Use `--profile-file path/to/profile.json` for a custom manifest. Existing files are preserved or merged according to the profile merge policy; generated artifacts should be changed through their source templates or generators.

## Ratchet After Baseline

Only ratchet after the team accepts a baseline score:

```bash
jankurai audit . --mode advisory --json target/jankurai/baseline-score.json --md target/jankurai/baseline-score.md
jankurai ci install . --github --mode ratchet --baseline target/jankurai/baseline-score.json
```

Ratchet mode blocks regression against the baseline. It is intentionally separate from `init --level ci`.

## Common Commands

```bash
jankurai adopt . --mode observe --out target/jankurai/adoption-plan.json --md target/jankurai/adoption-plan.md
jankurai doctor . --fail-on high --json target/jankurai/doctor.json --md target/jankurai/doctor.md
jankurai lane . --changed README.md --out target/jankurai/proof-plan.json --md target/jankurai/proof-plan.md
jankurai prove . --changed README.md --plan-out target/jankurai/proof-plan.json --plan-md target/jankurai/proof-plan.md
jankurai proof-verify . --plan target/jankurai/proof-plan.json --evidence-index target/jankurai/evidence-index.json --out target/jankurai/proof-verify.json --md target/jankurai/proof-verify.md
jankurai repair-plan . --from target/jankurai/repo-score.json --out target/jankurai/repair-plan.json --md target/jankurai/repair-plan.md
jankurai repair . --plan target/jankurai/repair-plan.json --dry-run --out target/jankurai/repair-run.json --md target/jankurai/repair-run.md
```

More surfaces:

```bash
jankurai context-pack . --task "tighten README install docs" --changed README.md --out target/jankurai/context-pack.json --md target/jankurai/context-pack.md
jankurai registry . --out target/jankurai/cell-registry.json --md target/jankurai/cell-registry.md
jankurai cell . --cell-id background-job --mode prove --out target/jankurai/background-job.json --md target/jankurai/background-job.md
jankurai migrate . --analyze --out target/jankurai/migration-report.json --md target/jankurai/migration-report.md
jankurai security run . --out target/jankurai/security/evidence.json
jankurai exceptions expire . --strict --out target/jankurai/exceptions.json --md target/jankurai/exceptions.md
jankurai certify . --out target/jankurai/certification.json --md target/jankurai/certification.md
jankurai govern . --out target/jankurai/governance.json --md target/jankurai/governance.md
jankurai bench . --out target/jankurai/benchmark.json --md target/jankurai/benchmark.md
jankurai publish . --certification target/jankurai/certification.json --benchmark target/jankurai/benchmark.json --governance target/jankurai/governance.json --out target/jankurai/publication.json --md target/jankurai/publication.md
```

Adapter and UX helpers:

```bash
jankurai adapters sync . --ide all --dry-run
jankurai adapters verify .
jankurai agent verify .
jankurai ux --help
```

## What Lives Here

- `crates/jankurai/` - Rust audit CLI, init, proof, repair, migration, and publication logic
- `packages/ux-qa/` - rendered-UX geometry and accessibility checks
- `agent/` - owner map, test map, generated-zone manifest, proof lanes, version bindings
- `docs/` - mission, standard, release, testing, install, and architecture notes
- `paper/` - canonical paper source and generated PDF
- `tips/` - phase notes and source material
- `reference/` - read-only source material

## Source Workspace Validation

```bash
just fast
just score
just ux-qa
just paper
just check
```

Generated outputs stay in declared generated zones. `reference/` stays read-only. The repo should remain clean enough that a fresh agent can locate ownership, choose the smallest proof lane, and leave a verifiable repair trail.
