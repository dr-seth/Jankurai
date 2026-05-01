# humanlint

<img src="assets/humanlint_readme.png" alt="Humans Were the Bug" width="100%">

`humanlint` is an open-source standard, paper, and audit CLI for agent-native engineering.

The thesis is simple: codebases optimized for human memory and vibe-coded momentum are now the liability. Agent-era repositories need strict ownership, generated contracts, small proof lanes, bounded language roles, agent-friendly exceptions, and CI findings that a coding agent can repair directly.

## Install

From this checkout:

```bash
./install.sh
humanlint /path/to/repo --json repo-score.json --md repo-score.md
```

With standard Python packaging:

```bash
python3 -m pip install .
humanlint /path/to/repo --json repo-score.json --md repo-score.md
```

Without installing:

```bash
python3 tools/humanlint.py /path/to/repo --json repo-score.json --md repo-score.md
```

`humanlint` is currently a dependency-free Python 3 CLI so any repository can audit itself without a bootstrap step. The performance path for the project is a Rust core scanner with the same CLI contract; the Python implementation is intentionally boxed as a tool, not product runtime.

## Paper

The white paper builds to:

```text
paper/humanlint.pdf
```

Build it with:

```bash
just paper
```

The source is [paper/humanlint.tex](paper/humanlint.tex). The Markdown companion is [paper/humanlint.md](paper/humanlint.md).

## Standard

The target stack is:

- Rust core
- TypeScript/React/Vite product surface
- PostgreSQL durable truth
- generated contracts
- bounded Python AI/data service only

The portable bootstrap for adopting repositories is [agent/HUMANLINT_STANDARD.md](agent/HUMANLINT_STANDARD.md). The full operating standard is [docs/agent-native-standard.md](docs/agent-native-standard.md).

## Audit

Run the audit in CI on every pull request:

```bash
humanlint . --json repo-score.json --md repo-score.md
```

The JSON output is the machine contract. The Markdown output is the review surface. Every finding is designed to include concrete evidence and an agent repair action.

Hard failures include:

- no root agent instructions
- no one-command validation
- no deterministic fast lane
- missing security lane
- generated contract drift
- Python owning product truth
- handwritten API mirrors
- direct DB access from the wrong layer
- duplicated logic
- fallback soup
- future-hostile terms such as `legacy`, `deprecated`, `temporary`, `stub`, and `TODO` in product/runtime code

## Repository Map

- `humanlint/` - installable CLI package
- `tools/humanlint.py` - checkout-local compatibility launcher
- `agent/` - versioned standard metadata, proof lanes, owner/test maps
- `docs/` - standard, mission, audit rubric, boundaries, testing, release plan
- `paper/` - IEEE-style white paper source, figures, bibliography, and PDF
- `assets/` - README and paper graphics
- `reference/` - preserved prior research assets used as source material

## Development

```bash
just paper
python3 -m py_compile tools/humanlint.py
```

Do not hand-edit generated files. Do not run broad audits as a substitute for targeted tests when changing the CLI.
