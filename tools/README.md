# humanlint Repo Scorer

Use `tools/humanlint.py` or the installed `humanlint` command to score a repository against humanlint `0.2.0`.

Target stack only: Rust core + TypeScript/React/Vite + PostgreSQL + generated contracts + bounded Python AI/data service. This is not a generic linter.

## Contract

- Fast and dependency-free Python 3 script
- No third-party packages
- Runs on arbitrary checkouts without bootstrap drama
- Emits JSON and Markdown
- Reads repo structure and local evidence, not build artifacts
- Does not mutate the target repo
- Produces actionable `agent_fix_queue` items
- Carries hard caps for severe agent-native failures

## Usage

```bash
humanlint /path/to/repo --json repo-score.json --md repo-score.md
python3 tools/humanlint.py /path/to/repo --json repo-score.json --md repo-score.md
python3 tools/humanlint.py /path/to/repo --changed src/foo.rs contracts/api.yaml
```

Install the checkout-local command with:

```bash
./install.sh
```

Or install the package entrypoint:

```bash
python3 -m pip install .
humanlint /path/to/repo --json repo-score.json --md repo-score.md
```

## CI

Run the scorer in every PR:

```bash
python3 tools/humanlint.py . --json repo-score.json --md repo-score.md
```

Upload both files. The JSON is the machine contract; the Markdown is the human review surface. Teams can fail CI on score, caps, or selected severities.

## Output

- `standard_version`
- `target_stack`
- `score`
- `raw_score`
- `caps_applied`
- `hard_rules`
- `dimensions`
- `findings`
- `agent_fix_queue`

## Strict Checks

The scorer flags known vibe-coding failure modes:

| Category | Hard Signals |
| --- | --- |
| stack drift | unnecessary runtime languages, too much Python, Python outside `python/ai-service` |
| code shape | duplication, mega files, mega functions, weak names, junk drawers |
| placeholders | TODO/FIXME/HACK/XXX, stubs, placeholders, unimplemented/unreachable/TODO panics |
| fallbacks | fallback soup, best-effort retries, broad catch/except, null/undefined fallbacks |
| contracts | handwritten DTO/API types, handwritten web API clients, missing generated clients, drift untested |
| generated zones | missing generated-zone manifest, missing do-not-edit markers, TODOs in generated code |
| data | direct DB access from web/API/domain/application/Python product code |
| tests | no Playwright/e2e for web, no Rust property tests, no Rust integration tests |
| security | no security lane, no secret scan, no dependency/SBOM/provenance scan |
| docs | missing root instructions, missing architecture/boundary/testing docs |
| exceptions | no agent-friendly errors with name/code/purpose/reason/common fixes/docs URL |

See `docs/audit-rubric.md` for the full scoring contract.
