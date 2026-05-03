# Installing jankurai

## Profiles

Bundled init profiles are defined as JSON validated against `schemas/init-profile.schema.json`.

- **`rust-ts-postgres`** (aliases: `rust-ts-vite-react-postgres`, `rust-ts-vite-react-postgres-bounded-python`) is the default full scaffold: agent constitution, IDE adapters, `contracts/` and `db/` README slots, `docs/architecture/` and `docs/decisions/` stubs, and `tools/security-lane.sh` stub.
- **`rust-api`**, **`react-web`**, **`b2b-saas`**, **`ai-product`**, **`regulated-saas`**, **`migration-target`** ship as bundled manifests under `crates/jankurai/templates/profiles/`.
- **`--profile-file path/to/profile.json`** loads a repo-local or shared manifest (same schema). Bundled `--profile` is not used to resolve the manifest when this flag is set. Plan JSON uses the manifest **`id`** as **`profile`**.
- Unknown bundled `--profile` values fail fast with an error listing supported IDs.

The canonical default manifest is `crates/jankurai/templates/profiles/rust-ts-postgres.json`. Planned file actions in dry-run / plan JSON are exactly the paths in `generatedPaths` (sorted); each path must have a matching entry in `crates/jankurai/src/init/templates.rs`.

Dry-run first:

```bash
jankurai init --profile rust-ts-vite-react-postgres --ide all --mode advisory --dry-run
```

Apply when the plan looks right:

```bash
jankurai init --profile rust-ts-vite-react-postgres --ide all --mode advisory --yes
jankurai doctor --fail-on high
jankurai ci install --github --mode ratchet --min-score 85
jankurai agent verify
```

`init --yes` creates missing paths from the profile and **merges** into some existing files instead of overwriting:

- **`.json`**: additive object/array merge (`merge-json`).
- **`.toml`**: additive table/array merge (`merge-toml`).
- **`Justfile`** or **`.gitignore`**: append lines from the template that are not already present (`merge-lines`).
- **`AGENTS.md`** and **`agent/JANKURAI_STANDARD.md`**: append an HTML merge marker for manual review (`merge-marker`).
- **Other paths** that already exist: left unchanged (`keep-existing`).

Run **`jankurai init ... --dry-run`** (or **`--plan-json`**) first; the printed plan lists the action for each `generatedPaths` entry.

For agent repair work, use the narrow packet commands:

```bash
jankurai context-pack --task "repair agent context routing" --out target/jankurai/context-pack.json
jankurai repair-plan --from agent/repo-score.json --out target/jankurai/repair-plan.json
```
