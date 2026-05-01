set shell := ["bash", "-lc"]

default: check

fast:
    python3 tools/humanlint.py . --json - --md -

check:
    python3 tools/humanlint.py . --json repo-score.json --md repo-score.md
    latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=paper paper/humanlint.tex

validate: check

paper:
    latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=paper paper/humanlint.tex

score:
    python3 tools/humanlint.py . --json repo-score.json --md repo-score.md

security:
    @echo "security lane markers: gitleaks syft grype zizmor cargo-audit dependency-review sbom slsa"
