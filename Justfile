set shell := ["bash", "-lc"]

default: check

fast:
    cargo check -p jankurai
    cargo run -p jankurai -- . --json target/jankurai/fast-score.json --md target/jankurai/fast-score.md

setup:
    npm ci

versions:
    cargo run -p jankurai -- versions

ux-qa:
    npm --workspace @jankurai/ux-qa run build
    npm --workspace @jankurai/ux-qa run test

check:
    cargo run -p jankurai -- versions
    cargo run -p jankurai -- . --json agent/repo-score.json --md agent/repo-score.md
    latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=paper paper/jankurai.tex

validate: check

paper:
    latexmk -pdf -interaction=nonstopmode -halt-on-error -outdir=paper paper/jankurai.tex

score:
    cargo run -p jankurai -- . --json agent/repo-score.json --md agent/repo-score.md

self-audit:
    cargo run -p jankurai -- audit . --self-audit --json target/jankurai/self-audit.json --md target/jankurai/self-audit.md

security:
    bash tools/security-lane.sh
