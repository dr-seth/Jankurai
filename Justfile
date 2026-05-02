set shell := ["bash", "-lc"]

default: check

fast:
    cargo check -p humanlint
    cargo run -p humanlint -- . --json target/humanlint/fast-score.json --md target/humanlint/fast-score.md

setup:
    npm ci

versions:
    cargo run -p humanlint -- versions

ux-qa:
    npm --workspace @humanlint/ux-qa run build
    npm --workspace @humanlint/ux-qa run test

check:
    cargo run -p humanlint -- versions
    cargo run -p humanlint -- . --json agent/repo-score.json --md agent/repo-score.md
    latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=paper paper/humanlint.tex

validate: check

paper:
    latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=paper paper/humanlint.tex

score:
    cargo run -p humanlint -- . --json agent/repo-score.json --md agent/repo-score.md

self-audit:
    cargo run -p humanlint -- audit . --self-audit --json target/humanlint/self-audit.json --md target/humanlint/self-audit.md

security:
    bash tools/security-lane.sh
