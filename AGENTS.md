# humanlint Agent Instructions

Read `agent/HUMANLINT_STANDARD.md` first. The full standard is in
`docs/agent-native-standard.md`; the paper mission is in `docs/mission.md`.

This workspace is writing and validating the paper:
`Humans Were the Bug: From Vibe Coding to Agent-Native Engineering`.

## Rules

- Keep all new files under `/Users/bentaylor/Code/humanlint`.
- Treat `reference/` as read-only source material.
- Do not hand-edit generated artifacts unless the generator/source is changed.
- Keep root guidance short; put durable detail in `docs/` or `agent/`.
- Use `python3 tools/humanlint.py . --json repo-score.json --md repo-score.md`
  for the audit lane.
- Use `latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=paper paper/humanlint.tex`
  for the paper lane.

## Validation

- Fast: `just fast`
- Audit: `just score`
- Paper: `just paper`
- Full: `just check`
