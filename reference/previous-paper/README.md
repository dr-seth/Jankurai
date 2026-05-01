# Paper Workspace

Start with [`main.md`](./main.md) for navigation across the paper family. The canonical typeset source is [`main.tex`](./main.tex).

## Audiences

| If you are a… | Read |
|---|---|
| Reviewer of the paper | `main.tex` (camera-ready source) or `paper-detailed.md` (extended Markdown narrative with richer examples) |
| Coding agent needing a tight brief | `paper-for-agents.md` (~500 words) |
| Executive / sponsor | `executive-brief.md` (CTO-oriented summary) |
| Claim verifier | `citation-index.md` (claims → evidence class) |

## Contents of this folder

| File | Role |
|---|---|
| `main.tex` | Submission-facing IEEE LaTeX manuscript; source of truth. |
| `main.md` | Navigation hub; points readers at the right companion. |
| `main.pdf` | Compiled manuscript (run `latexmk -pdf main.tex` to rebuild). |
| `paper-detailed.md` | Full Markdown narrative with extended examples and artifact links. |
| `paper-for-agents.md` | Compressed ~500-word digest for agent consumption. |
| `executive-brief.md` | 1-page CTO summary. |
| `citation-index.md` | Per-claim evidence index (evidence-backed / mechanism / project claim / doctrine). |
| `references.bib` | BibTeX database. |
| `artifacts/` | Supplement: tooling catalog, token-saving patterns, concept specs for the three flagship future systems, upstream-horizon concepts, reference workspace tree. |
| `build/` | Preview build artifacts (not source). |
| `generated/` | Historical generated tables (`paper-sync` output). |

## Maintenance flow

1. Update the ledger (`/docs/research/claim-citation-ledger.md`) first.
2. Update `main.tex`.
3. Propagate changes to `paper-detailed.md`, then regenerate `paper-for-agents.md` and `executive-brief.md` from `main.tex`.
4. Run `cargo run -p paper-sync -- index` and `cargo run -p paper-sync -- check`.
5. Rebuild with `latexmk -pdf -interaction=nonstopmode -halt-on-error main.tex`.

## V7 commitments

- The main paper is a review and operating standard, not a local empirical benchmark paper.
- The flagship future-work trio is `cargo-mss`, ProofLens, and `cargo-obligation-cache`.
- Exhaustive tooling catalogs, token-saving patterns, and secondary concepts live in `artifacts/` and `paper-detailed.md`, not in the main manuscript.
