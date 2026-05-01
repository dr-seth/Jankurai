# humanlint Testing

Testing is routed proof. Agents should not guess which tests matter.

| Lane | Purpose |
| --- | --- |
| `fast` | deterministic local proof for most edits |
| `contract` | API/schema generation and drift checks |
| `db` | migrations, constraints, schema drift |
| `web` | TypeScript typecheck, component tests, rendered UX QA |
| `e2e` | Playwright critical product flows |
| `security` | secrets, dependencies, SBOM/SCA, workflow lint |
| `observability` | traces, request IDs, structured error payloads |
| `audit` | humanlint repo score and hard-rule findings |
| `full` | release/merge gate |

For this workspace:

- `just versions` checks version and artifact bindings.
- `just ux-qa` builds and tests the optional Playwright geometry runtime.
- `just fast` runs the scorer on stdout.
- `just score` writes `repo-score.json` and `repo-score.md`.
- `just paper` builds `paper/humanlint.pdf`.
- `just check` runs version checks, audit, and paper build.

Rendered UX QA combines Storybook states, Playwright screenshots, visual review, axe/WCAG checks, CLS checks, MSW/generated mocks, design tokens, and deterministic DOM geometry rules such as edge clearance, target size, overlap, clipping, wrapping, horizontal overflow, sticky obstruction, focus visibility, form labels, and nested scrollbars.

The full target-stack test doctrine lives in `docs/agent-native-standard.md`
and the canonical TeX paper under `paper/tex/`.
