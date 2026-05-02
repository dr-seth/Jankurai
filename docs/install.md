# Installing humanlint

## Profiles

Bundled init profiles are defined as JSON validated against `schemas/init-profile.schema.json`.

- **`rust-ts-postgres`** (aliases: `rust-ts-vite-react-postgres`, `rust-ts-vite-react-postgres-bounded-python`) is the default full scaffold: agent constitution, IDE adapters, `contracts/` and `db/` README slots, `docs/architecture/` and `docs/decisions/` stubs, and `tools/security-lane.sh` stub.
- Any other `--profile` value fails fast with an error listing supported IDs.

The canonical manifest is shipped at `crates/humanlint/templates/profiles/rust-ts-postgres.json` in this repository. Planned file actions in dry-run / plan JSON are exactly the paths in `generatedPaths` (sorted); each path must have a matching entry in `crates/humanlint/src/init/templates.rs`.

Dry-run first:

```bash
humanlint init --profile rust-ts-vite-react-postgres --ide all --mode advisory --dry-run
```

Apply when the plan looks right:

```bash
humanlint init --profile rust-ts-vite-react-postgres --ide all --mode advisory --yes
humanlint doctor --fail-on high
humanlint ci install --github --mode ratchet --min-score 85
humanlint agent verify
```

`init --yes` creates missing files and leaves existing user content intact. When an
existing canonical file is not already humanlint-controlled, it appends a merge
marker instead of overwriting.

For agent repair work, use the narrow packet commands:

```bash
humanlint context-pack --task "repair agent context routing" --out target/humanlint/context-pack.json
humanlint repair-plan --from agent/repo-score.json --out target/humanlint/repair-plan.json
```
