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

compat:
    cargo test -p jankurai --test report_compatibility_guard

self-audit:
    cargo run -p jankurai -- audit . --self-audit --json target/jankurai/self-audit.json --md target/jankurai/self-audit.md

security:
    cargo run -p jankurai -- security run . --out target/jankurai/security/evidence.json

security-strict:
    cargo run -p jankurai -- security run . --strict --out target/jankurai/security/evidence.json

security-bash:
    bash tools/security-lane.sh

phase12:
    mkdir -p target/jankurai/public
    cargo run -p jankurai -- bench . --out target/jankurai/p12-benchmark-report.json --md target/jankurai/p12-benchmark-report.md
    cargo run -p jankurai -- certify . --out target/jankurai/p12-certification.json --md target/jankurai/p12-certification.md
    cargo run -p jankurai -- govern . --out target/jankurai/p12-governance-policy.json --md target/jankurai/p12-governance-policy.md
    cargo run -p jankurai -- publish . --certification target/jankurai/p12-certification.json --benchmark target/jankurai/p12-benchmark-report.json --governance target/jankurai/p12-governance-policy.json --out target/jankurai/public/p12-public-evidence.json --md target/jankurai/public/p12-public-evidence.md --badge-json target/jankurai/public/jankurai-badge.json --badge-svg target/jankurai/public/jankurai-badge.svg

phase13:
    mkdir -p target/jankurai
    cargo run -p jankurai -- optimize . --mode all --out target/jankurai/p13-optimization-report.json --md target/jankurai/p13-optimization-report.md
    cargo run -p jankurai -- exceptions expire . --warning-days 7 --strict --out target/jankurai/p13-exception-expiry.json --md target/jankurai/p13-exception-expiry.md
