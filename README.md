# humanlint

<img src="assets/humanlint_readme.png" alt="Humans Were the Bug" width="100%">

`humanlint` is an open-source standard, paper, and audit CLI for agent-native engineering: repositories built so coding agents can reject wrong code, localize the repair, run the right proof lane, and leave evidence.

Use it in two steps: audit now, then ratchet toward conformance. The audit works on any repo; the default target profile is Rust core, TypeScript/React/Vite product surface, PostgreSQL durable truth, generated contracts, and bounded Python for AI/data service work.

<img src="assets/vibe-coding-tlr-pie.svg" alt="Vibe-coding top-level risk shares" width="100%">

## Vibe-Coding Risk Model

`TLR` means Top-Level Risk. These shares are policy-weighted RPN from the paper taxonomy, not incident-frequency measurements.

| TLR | Share | Highest-risk faults | Primary controls |
| --- | ---: | --- | --- |
| Security | 32% | generated insecure code, secrets, PII, prompt injection, excessive agency | security lane, SAST/SCA, secret scans, permission receipts |
| Business truth | 20% | false-green rules, authz/data isolation, idempotency drift | domain invariants, role matrix tests, DB constraints, replay tests |
| Contracts/data | 13% | DTO drift, wrong-layer persistence, destructive migrations | generated clients, generated zones, DB migration proof |
| Verification | 13% | weak tests, pixel/UI, accessibility, eval drift | semantic assertions, Playwright, ARIA/axe, geometry reports |
| Entropy | 10% | dead language, orphan code, mega functions, perf drift | marker scan, owner review, LOC caps, benchmarks |
| Context/setup | 9% | context retrieval, setup hallucination, instruction drift | root router, owner/test maps, one-command setup |
| Repair | 3% | opaque exceptions and missing production evidence | OTel, problem details, repair receipts |

## Quick Start

Install the audit CLI from GitHub:

```bash
python3 -m pip install "git+https://github.com/jeppsontaylor/humanlint.git"
humanlint . --json repo-score.json --md repo-score.md
```

From a checkout:

```bash
git clone https://github.com/jeppsontaylor/humanlint.git
cd humanlint
python3 -m pip install .
humanlint /path/to/repo --json repo-score.json --md repo-score.md
```

Without installing:

```bash
python3 tools/humanlint.py /path/to/repo --json repo-score.json --md repo-score.md
```

The Python audit path is dependency-free. It emits one machine-readable JSON report and one Markdown review surface.

Read the Markdown report first. Fix the highest-priority `agent_fix_queue` item, rerun the audit, and repeat until caps are gone. Scores below `70` usually mean the repo is not agent-operable; `70-84` is advisory/ratchet territory; `85+` is the target floor for standard-mode conformance.

## Add The Agent Standard

Download the short agent bootstrap into your repo:

```bash
mkdir -p agent
curl -fsSL https://raw.githubusercontent.com/jeppsontaylor/humanlint/main/agent/HUMANLINT_STANDARD.md \
  -o agent/HUMANLINT_STANDARD.md
```

`wget` equivalent:

```bash
mkdir -p agent
wget -qO agent/HUMANLINT_STANDARD.md \
  https://raw.githubusercontent.com/jeppsontaylor/humanlint/main/agent/HUMANLINT_STANDARD.md
```

If you do not already have `AGENTS.md`, create one:

```bash
cat > AGENTS.md <<'EOF'
# Agent Instructions

Read `agent/HUMANLINT_STANDARD.md` first.

Run the smallest mapped validation lane before final response.
Do not edit generated files by hand.
Keep durable project detail in `docs/` or `agent/`, not root prose.
EOF
```

If your repo already has `AGENTS.md`, add this line near the top instead:

```md
Read `agent/HUMANLINT_STANDARD.md` first.
```

Then run the audit and fix the highest-severity findings first:

```bash
humanlint . --json repo-score.json --md repo-score.md
```

For real conformance, add the machine-readable maps the audit expects:

| File | Purpose |
| --- | --- |
| `agent/owner-map.json` | maps changed paths to ownership cells |
| `agent/test-map.json` | maps changed paths to proof lanes |
| `agent/generated-zones.toml` | names generated outputs, sources, and regeneration commands |
| `agent/standard-version.toml` | pins standard, audit, schema, paper, and artifact versions |

## CI

GitHub Actions starter:

```yaml
name: humanlint

on:
  pull_request:
  push:
    branches: [main]

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: "3.12"
      - run: python3 -m pip install "git+https://github.com/jeppsontaylor/humanlint.git"
      - run: humanlint . --json repo-score.json --md repo-score.md
      - uses: actions/upload-artifact@v4
        with:
          name: humanlint-score
          path: |
            repo-score.json
            repo-score.md
```

Teams usually start in advisory mode, then ratchet toward a score floor of `85` and no high-severity findings without an exception.

## Rendered UX QA

Existing open-source tools solve slices of frontend QA: Storybook states, Playwright screenshots, Argos/BackstopJS visual review, axe accessibility scans, Lighthouse/Web Vitals, MSW mocks, and design tokens. Humanlint adds the missing deterministic geometry layer: target size, edge clearance, overlap, clipping, button wrapping, horizontal overflow, sticky obstruction, focus visibility, form labels, nested scrollbars, and z-index token checks.

From this repo:

```bash
npm ci
npx playwright install chromium
npm --workspace @humanlint/ux-qa run build
npm --workspace @humanlint/ux-qa run test
```

Library use in Playwright:

```ts
import { test } from "@playwright/test";
import { expectNoUxViolations } from "@humanlint/ux-qa";

test("dashboard rendered UX", async ({ page }) => {
  await page.goto("http://localhost:3000/dashboard");
  await expectNoUxViolations(page);
});
```

CLI use after building the workspace package:

```bash
node packages/ux-qa/dist/cli.js audit \
  --url http://localhost:3000 \
  --out ux-qa.json \
  --route-id dashboard \
  --artifacts-dir ux-qa-artifacts \
  --screenshot \
  --aria-snapshot \
  --viewport 390x844 \
  --viewport 1440x900
```

The CLI expects a running app or preview URL. Reports include rule IDs, selectors, viewport data, severity, merge decision, and artifact paths for screenshots, crops, and ARIA snapshots when requested. Deterministic geometry failures should block; visual baseline diffs should route to owner approval; AI/VLM visual opinions should stay review-only unless backed by deterministic evidence.

## Repository Map

- `humanlint/` - installable audit package
- `tools/humanlint.py` - checkout-local launcher
- `packages/ux-qa/` - optional Playwright rendered-UX geometry runtime
- `agent/` - standard version, owner/test maps, generated zones, proof lanes
- `docs/` - standard, rubric, testing doctrine, architecture, release plan
- `paper/` - IEEE-style paper source and generated PDF
- `tips/` - source notes feeding the paper and standard
- `reference/` - read-only source material

## Paper Naming

Current paper artifacts are intentionally named after the project:

- `paper/humanlint.tex` - canonical TeX entrypoint
- `paper/humanlint.pdf` - generated render
- `paper/humanlint.md` - agent-readable companion
- `paper/tex/` - included TeX source sections

Do not add `main.md`, `main.tex`, or `main.pdf` anywhere in this repo. `just versions` enforces this so every paper artifact stays project-titled.

## Development

```bash
just fast
just ux-qa
just paper
just check
```

Do not hand-edit generated artifacts such as `paper/humanlint.pdf` or `package-lock.json`; change the source and regenerate.
