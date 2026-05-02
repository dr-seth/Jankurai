# Jankurai Agent Instructions

Read `agent/JANKURAI_STANDARD.md` first. For phase or MASTER_PLAN work,
read `agent/MASTER_PLAN.md` before `tips/phases/00-phase-index.md`. The full
standard is in `docs/agent-native-standard.md`; the paper mission is in
`docs/mission.md`.

This workspace is writing and validating the paper:
`Humans Were the Bug: From Vibe Coding to Agent-Native Engineering`.

## Rules

- Keep new files inside the repository root.
- Treat `reference/` as read-only source material.
- Do not hand-edit generated artifacts unless the generator/source is changed.
- Keep root guidance short; put durable detail in `docs/` or `agent/`.
- Use `cargo run -p jankurai -- . --json agent/repo-score.json --md agent/repo-score.md`
  for the audit lane.
- Use `latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=paper paper/jankurai.tex`
  for the paper lane.

## Validation

- Fast: `just fast`
- Audit: `just score`
- Paper: `just paper`
- Full: `just check`
