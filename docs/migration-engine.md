# Migration Engine

Jankurai provides a measured strangler migration engine that analyzes legacy codebases, computes liability scores, and produces schema-validated migration plans.

## Philosophy

Migration is **not rewriting**. It is:

```text
discover → score → isolate contracts → build harness → port one slice → prove equivalence → cut over → retire old code
```

Every migration plan is `dry-run` by default. Execution remains bounded and requires human approval for high-risk cutovers.

## Commands

### Analyze

Scans a repository and produces a `MigrationReport` with stack detection, liability scoring, and strangler candidates:

```bash
jankurai migrate --analyze . --out target/jankurai/migration-report.json --md target/jankurai/migration-report.md
```

### Plan

Produces a `MigrationPlan` with concrete slices derived from the analysis:

```bash
jankurai migrate . --out target/jankurai/migration-plan.json --md target/jankurai/migration-plan.md
```

## Stack Detection

The engine detects the following from filesystem heuristics (manifest files, not code parsing):

| Marker | Detected As |
|--------|------------|
| `Cargo.toml` | Rust, cargo, cargo-test |
| `package.json` | TypeScript, npm |
| `requirements.txt` / `pyproject.toml` | Python, pip, pytest |
| `pom.xml` / `build.gradle` | Java, maven/gradle, junit |
| `Gemfile` | Ruby, bundler, rspec, rails |
| `composer.json` | PHP, composer |
| `go.mod` | Go, go-modules, go-test |
| `.github/workflows/` | github-actions CI |

Framework and DB client detection reads manifest content (lowercase contains) for known markers like `actix`, `express`, `fastapi`, `sqlx`, `prisma`, etc.

## Liability Score

The liability score (0–100) estimates migration risk:

| Factor | Effect |
|--------|--------|
| No lockfile | +10 |
| No test framework | +10 |
| No CI system | +5 |
| Multiple DB clients | +5 |
| Polyglot (>2 languages) | +5 |
| Rust is primary language | -10 |
| Test framework present | -5 |
| Base | 50 |

Scores above 70 trigger mandatory human approval for all migration slices.

## Known Limitations

- Stack detection is file-existence based, not AST-based. It can miss languages used only in subdirectories.
- Liability scoring is heuristic. It does not measure actual code complexity.
- Contract extraction and equivalence proofs are documented as slice types but are not executed automatically.
- The engine does not modify source code. It produces plans only.
