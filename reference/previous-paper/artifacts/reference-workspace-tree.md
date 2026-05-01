# Reference Workspace Tree

Status: supplement artifact for the V7 review-backed standard.

```text
AGENTS.md
proof-lanes.toml
token-budget.toml
.cargo/config.toml
Cargo.toml
crates/
  domain/         # pure invariants and closed state
  application/    # use cases and orchestration
  adapters/       # DB, HTTP clients, queues, files, third-party APIs
  api/            # HTTP or RPC boundary, schemas, handlers
  cli/            # operator-facing commands
docs/
  contracts/      # generated schemas and contract docs
  repo-map/       # owner map, test map, contract map if generated
tests/
  e2e/            # black-box flows and abuse cases
fuzz/             # fuzz targets when input risk exists
xtask/            # repository automation and generation tasks
```

## Notes

- Keep generated artifacts separate from handwritten logic.
- Keep domain invariants out of adapters.
- Keep the root `AGENTS.md` short and routing-oriented.
- Put task-specific or crate-specific guidance in local docs instead of bloating the root instructions.
