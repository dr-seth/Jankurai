<p align="center">
  <img src="assets/jankurai_header.png" alt="Jankurai" width="100%">
</p>

<p align="center">
  <strong>
    <font color="green">Level 1: zero-risk reporting</font>
    &nbsp; | &nbsp;
    <font color="blue">Level 2: agent guidance</font>
    &nbsp; | &nbsp;
    <font color="orange">Level 3: full control-plane commit</font>
    &nbsp; | &nbsp;
    <font color="red">Ratchet: opt-in enforcement</font>
  </strong>
</p>

# Jankurai

Jankurai is a control plane for agent-native repositories. It turns agent
guidance, ownership, proof lanes, generated zones, scoring, CI evidence, and
repair queues into files that agents and humans can both read.

The point is not to make code generation louder. The point is to make it easier
to reject bad changes, route narrow fixes, prove the right lane, and leave a
repair trail the next agent can use.

```text
Level 1 report -> Level 2 guide agents -> Level 3 full commit -> ratchet later
```

## Install

Prerequisites: `git` and a Rust toolchain with `cargo` on `PATH`.

```bash
git clone https://github.com/jeppsontaylor/Jankurai.git
cd Jankurai
cargo install --path crates/jankurai --locked
jankurai --version
```

Then move to the repository you want to inspect or adopt:

```bash
cd /path/to/your/repo
```

For colorful terminal output and visible progress in recordings, demos, or CI
logs, force rich output:

```bash
export JANKURAI_COLOR=always
export JANKURAI_PROGRESS=always
```

## Choose Your Risk Level

<table>
  <thead>
    <tr>
      <th align="left">Level</th>
      <th align="left">Risk</th>
      <th align="left">What it does</th>
      <th align="left">What it will not do</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><strong><font color="green">Level 1</font></strong><br><code>observe</code></td>
      <td>Reads the repo and writes reports under <code>target/jankurai/</code>.</td>
      <td>Audits, scores, detects adoption gaps, and produces an agent-readable workplan.</td>
      <td>No tracked files, no agent instructions, no CI, no commits.</td>
    </tr>
    <tr>
      <td><strong><font color="blue">Level 2</font></strong><br><code>guide</code></td>
      <td>Adds or merges agent-facing guidance files.</td>
      <td>Installs <code>AGENTS.md</code>, the Jankurai standard, master plan, and provider adapters for Codex, Cursor, Copilot, Claude, Gemini, and similar tools.</td>
      <td>No CI workflow, no score gate, no full scaffold.</td>
    </tr>
    <tr>
      <td><strong><font color="orange">Level 3</font></strong><br><code>full commit</code></td>
      <td>Creates the full control-plane scaffold on a branch and commits it.</td>
      <td>Adds ownership maps, test maps, proof lanes, generated-zone policy, docs, contracts/db placeholders, security policy, observe-mode CI tools, and an adoption workplan.</td>
      <td>No ratchet enforcement unless you explicitly install ratchet mode after accepting a baseline.</td>
    </tr>
  </tbody>
</table>

## Level 1: Zero-Risk Reporting

Use this when you want value before letting Jankurai change anything. These
commands read the repository and write reports under `target/jankurai/`.

```bash
mkdir -p target/jankurai

jankurai adopt . \
  --profile auto \
  --mode observe \
  --out target/jankurai/adoption-plan.json \
  --md target/jankurai/adoption-plan.md

jankurai doctor . \
  --fail-on critical \
  --json target/jankurai/doctor.json \
  --md target/jankurai/doctor.md

jankurai audit . \
  --mode advisory \
  --json target/jankurai/repo-score.json \
  --md target/jankurai/repo-score.md
```

What you get:

- `target/jankurai/adoption-plan.md`: what Jankurai would recommend next
- `target/jankurai/doctor.md`: missing files, health checks, and local blockers
- `target/jankurai/repo-score.md`: advisory score, findings, and repair queue
- `target/jankurai/score-history.jsonl`: one score-history row per audit run

Risk profile:

- Reads source files.
- Writes only the report paths you named.
- Does not create `AGENTS.md`.
- Does not touch `.github/workflows/`.
- Does not commit.

## Level 2: Agent Guidance

Use this when you want agents to know how to behave in the repo, but you do not
want CI or the full scaffold yet.

Preview first:

```bash
jankurai init . \
  --level agents \
  --dry-run \
  --plan-json target/jankurai/init-agents.json
```

Apply the small agent-facing install:

```bash
jankurai init . --level agents --yes
jankurai adapters verify .
```

Then open Codex, OpenCode, Claude, Cursor, Copilot, or another agent from the
same repo root and say:

```text
Read AGENTS.md, follow the jankurai standard, then run the proof lane for my change.
```

What Level 2 may write:

```text
AGENTS.md
agent/JANKURAI_STANDARD.md
agent/MASTER_PLAN.md
.agents/
.claude/
.cursor/
.github/copilot-instructions.md
.github/instructions/
CLAUDE.md
GEMINI.md
```

Safety behavior:

- Dry run writes no repo files except the `--plan-json` path you requested.
- Existing files are kept, merged, or marked according to the init plan.
- Existing workflows are not installed at this level.
- Ratchet gates are not installed.

## Level 3: Full Jankurai Commit

Use this when you are ready to add the full control plane and commit it as a
reviewable change. Start from a clean worktree.

```bash
git status --short
git switch -c chore/adopt-jankurai
```

Create an adoption workplan and preview the full scaffold:

```bash
jankurai adopt . \
  --profile rust-ts-postgres \
  --mode observe \
  --out agent/adoption-plan.json \
  --md agent/adoption-plan.md

jankurai init . \
  --profile rust-ts-postgres \
  --level full \
  --dry-run \
  --plan-json target/jankurai/init-full.json
```

Apply the full scaffold and install observe-mode CI:

```bash
jankurai init . \
  --profile rust-ts-postgres \
  --level full \
  --yes

jankurai ci install . --github --mode observe --dry-run
jankurai ci install . --github --mode observe

jankurai hooks install . --dry-run
jankurai hooks install . --yes
```

Generate the tracked advisory score, run local checks, and commit the Jankurai
adoption. The first adoption commit can skip hooks because it uses the score
generated below:

```bash
jankurai audit . \
  --mode advisory \
  --json agent/repo-score.json \
  --md agent/repo-score.md \
  --score-history agent/score-history.jsonl \
  --score-history-csv agent/score-history.csv

jankurai doctor . \
  --fail-on high \
  --json target/jankurai/doctor.json \
  --md target/jankurai/doctor.md

git status --short
git add -- \
  AGENTS.md CLAUDE.md GEMINI.md Justfile README-jankurai-scaffold.md \
  .agents .claude .cursor .github \
  agent contracts db docs tools
git diff --staged --stat
JANKURAI_SKIP_HOOKS=1 git commit -m "Adopt Jankurai control plane"
```

The fast path is the explicit YOLO command. It applies Level 3, installs
observe-mode CI and local scoring hooks, writes `agent/repo-score.*`, appends
score history, stages the worktree, and commits with score trailers:

```bash
jankurai init . \
  --profile rust-ts-postgres \
  --yolo
```

`--yolo` is intentionally not Level 1 or Level 2. It is the "commit it" path.

What Level 3 may write:

```text
AGENTS.md
Justfile
agent/
agent/adoption-plan.json
agent/adoption-plan.md
agent/repo-score.json
agent/repo-score.md
agent/score-history.jsonl
agent/score-history.csv
.agents/
.claude/
.cursor/
.github/instructions/
.github/workflows/jankurai.yml
contracts/
db/
docs/
tools/security-lane.sh
tools/jankurai-hooks/pre-commit
tools/jankurai-hooks/prepare-commit-msg
README-jankurai-scaffold.md
```

What the commit means:

- Agents now have a standard entrypoint, owner map, test map, proof lanes, and
  generated-zone policy.
- CI runs in observe mode by default and reports evidence.
- Local commits run advisory scoring, stage score artifacts, and append
  `Jankurai-*` commit trailers. Use `JANKURAI_SKIP_HOOKS=1 git commit ...` only
  when you need to bypass local hooks.
- The repository has a reviewable scaffold commit instead of an invisible local
  setup step.
- Score enforcement is still not enabled.

## Score History

Every audit appends a JSONL score-history row by default:

```bash
jankurai audit . \
  --mode advisory \
  --json target/jankurai/repo-score.json \
  --md target/jankurai/repo-score.md
```

Default history path:

```text
target/jankurai/score-history.jsonl
```

For a committed progress chart, write history under `agent/` and ask for CSV:

```bash
jankurai audit . \
  --mode advisory \
  --json agent/repo-score.json \
  --md agent/repo-score.md \
  --score-history agent/score-history.jsonl \
  --score-history-csv agent/score-history.csv
```

The JSONL and CSV rows include the commit, branch, dirty-worktree flag, score,
raw score, caps, finding counts, decision, scope, and report fingerprint. That
gives you plot-ready data for "pre-commit score -> agent commits -> score
improves" demos.

Disable history for one run:

```bash
jankurai audit . \
  --mode advisory \
  --json target/jankurai/repo-score.json \
  --md target/jankurai/repo-score.md \
  --no-score-history
```

## Ratchet Later

Ratchet is the enforcement step. Do it only after the team accepts a baseline.

```bash
jankurai audit . \
  --mode advisory \
  --json target/jankurai/baseline-score.json \
  --md target/jankurai/baseline-score.md

jankurai ci install . \
  --github \
  --mode ratchet \
  --baseline target/jankurai/baseline-score.json
```

Ratchet blocks regression against the baseline. It is intentionally separate
from Level 1, Level 2, and Level 3.

## What Happens After Init?

If you ran this:

```bash
jankurai init . --profile rust-ts-postgres --yes
```

do this next:

```bash
jankurai audit . \
  --mode advisory \
  --json agent/repo-score.json \
  --md agent/repo-score.md \
  --score-history agent/score-history.jsonl \
  --score-history-csv agent/score-history.csv

jankurai doctor . --fail-on high
```

Then start Codex, OpenCode, Claude, Cursor, or another agent from the repository
root and give it this prompt:

```text
Read AGENTS.md, follow the jankurai standard, then run the proof lane for my change.
```

The value path is:

```text
install guidance -> check health -> get score and repair queue -> let the agent work with proof
```

## Common Commands

Use these after you choose an adoption level.

```bash
jankurai lane . \
  --changed README.md \
  --out target/jankurai/proof-plan.json \
  --md target/jankurai/proof-plan.md

jankurai prove . \
  --changed README.md \
  --plan-out target/jankurai/proof-plan.json \
  --plan-md target/jankurai/proof-plan.md

jankurai proof-verify . \
  --plan target/jankurai/proof-plan.json \
  --evidence-index target/jankurai/evidence-index.json \
  --out target/jankurai/proof-verify.json \
  --md target/jankurai/proof-verify.md

jankurai repair-plan . \
  --from target/jankurai/repo-score.json \
  --out target/jankurai/repair-plan.json \
  --md target/jankurai/repair-plan.md

jankurai repair . \
  --plan target/jankurai/repair-plan.json \
  --dry-run \
  --out target/jankurai/repair-run.json \
  --md target/jankurai/repair-run.md
```

More surfaces:

```bash
jankurai context-pack . \
  --task "tighten README install docs" \
  --changed README.md \
  --out target/jankurai/context-pack.json \
  --md target/jankurai/context-pack.md

jankurai migrate . \
  --analyze \
  --out target/jankurai/migration-report.json \
  --md target/jankurai/migration-report.md

jankurai security run . \
  --out target/jankurai/security/evidence.json

jankurai exceptions expire . \
  --strict \
  --out target/jankurai/exceptions.json \
  --md target/jankurai/exceptions.md
```

Publication and governance:

```bash
jankurai certify . \
  --out target/jankurai/certification.json \
  --md target/jankurai/certification.md

jankurai govern . \
  --out target/jankurai/governance.json \
  --md target/jankurai/governance.md

jankurai bench . \
  --out target/jankurai/benchmark.json \
  --md target/jankurai/benchmark.md

jankurai publish . \
  --certification target/jankurai/certification.json \
  --benchmark target/jankurai/benchmark.json \
  --governance target/jankurai/governance.json \
  --out target/jankurai/publication.json \
  --md target/jankurai/publication.md
```

Adapter and UX helpers:

```bash
jankurai adapters sync . --ide all --dry-run
jankurai adapters verify .
jankurai agent verify .
jankurai ux --help
```

## What Lives Here

- `crates/jankurai/`: Rust audit CLI, init, proof, repair, migration, and publication logic
- `packages/ux-qa/`: rendered-UX geometry and accessibility checks
- `agent/`: owner map, test map, generated-zone manifest, proof lanes, version bindings
- `docs/`: mission, standard, release, testing, install, and architecture notes
- `paper/`: canonical paper source and generated PDF
- `tips/`: phase notes and source material
- `reference/`: read-only source material

## Source Workspace Validation

```bash
just fast
just score
just ux-qa
just paper
just check
```

Generated outputs stay in declared generated zones. `reference/` stays
read-only. The repo should remain clean enough that a fresh agent can locate
ownership, choose the smallest proof lane, and leave a verifiable repair trail.
