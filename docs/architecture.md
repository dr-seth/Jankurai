# humanlint Architecture

humanlint is a paper, standard, and audit workspace. The product standard it
defines is:

```text
Rust core + TypeScript/React/Vite product surface + PostgreSQL truth
+ generated contracts + bounded Python AI/data service
```

The canonical architecture is documented in:

- `docs/agent-native-standard.md`
- `agent/HUMANLINT_STANDARD.md`
- `paper/sections/04_winner_architecture.md`

Local workspace ownership:

| Path | Role |
| --- | --- |
| `paper/` | manuscript, TeX wrapper, figures, citation ledgers |
| `tools/` | dependency-free audit script |
| `docs/` | mission, standard, research, release, audit doctrine |
| `agent/` | machine-readable maps and agent bootstrap |
| `reference/` | read-only copied source corpus |
| `tips/` | short reusable guidance distilled from the paper |

Agents should prefer `agent/owner-map.json` and `agent/test-map.json` for
changes, then route to the smallest proof lane.
