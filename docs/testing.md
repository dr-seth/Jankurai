# humanlint Testing

Testing is routed proof. Agents should not guess which tests matter.

| Lane | Purpose |
| --- | --- |
| `fast` | deterministic local proof for most edits |
| `contract` | API/schema generation and drift checks |
| `db` | migrations, constraints, schema drift |
| `web` | TypeScript typecheck, component tests |
| `e2e` | Playwright critical product flows |
| `security` | secrets, dependencies, SBOM/SCA, workflow lint |
| `observability` | traces, request IDs, structured error payloads |
| `audit` | humanlint repo score and hard-rule findings |
| `full` | release/merge gate |

For this workspace:

- `just fast` runs the scorer on stdout.
- `just score` writes `repo-score.json` and `repo-score.md`.
- `just paper` builds `paper/humanlint.pdf`.
- `just check` runs audit plus paper build.

The full target-stack test doctrine lives in `docs/agent-native-standard.md`
and `paper/sections/07_exceptions_and_qa.md`.
