set shell := ["bash", "-lc"]

default: check

fast:
    cargo run -p humanlint -- . --json - --md -

setup:
    npm ci

versions:
    cargo run -p humanlint -- versions

ux-qa:
    npm --workspace @humanlint/ux-qa run build
    npm --workspace @humanlint/ux-qa run test

check:
    cargo run -p humanlint -- versions
    cargo run -p humanlint -- . --json repo-score.json --md repo-score.md
    latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=paper paper/humanlint.tex

validate: check

paper:
    latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=paper paper/humanlint.tex

score:
    cargo run -p humanlint -- . --json repo-score.json --md repo-score.md

security:
    @echo "security lane markers: gitleaks syft grype zizmor cargo-audit dependency-review sbom slsa"
