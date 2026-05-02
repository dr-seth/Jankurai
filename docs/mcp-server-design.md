# Model Context Protocol (MCP) Server Design

This document details the intended future state of the Humanlint MCP Server surface.

## Philosophy

Humanlint is currently a CLI-first tool. The CLI is the single source of truth for planning, validation, and auditing. We will not overbuild the MCP server surface until the foundational contracts are locked in Phase 08.

**The CLI remains the source of truth.** No long-running server is required for local CLI execution.

## Candidate MCP Resources

Once implemented, the MCP server will expose the following immutable or frequently accessed metadata as Resources:

- **`humanlint://report`**: The current `agent/repo-score.json` or equivalent generated audit.
- **`humanlint://findings`**: A filtered view of open findings from the latest report.
- **`humanlint://owner-map`**: The contents of `agent/owner-map.json`.
- **`humanlint://test-map`**: The contents of `agent/test-map.json`.
- **`humanlint://generated-zones`**: The contents of `agent/generated-zones.toml`.
- **`humanlint://context-pack/{task_id}`**: The latest generated context pack.
- **`humanlint://proof-plan/{task_id}`**: The latest generated proof plan.

## Candidate MCP Tools

The MCP server will expose the following agent-executable commands as Tools:

- **`humanlint_audit`**: Trigger a fresh `humanlint . --json` audit.
- **`humanlint_lane_plan`**: Execute `humanlint lane . --changed <path>` to dynamically generate a proof plan for the current working tree.
- **`humanlint_context_pack`**: Trigger `humanlint context-pack --task "<task>"` and return the context bounds.
- **`humanlint_repair_plan`**: Trigger `humanlint repair-plan` to extract fix queues from the latest report.
- **`humanlint_explain_rule`**: Return the detailed documentation and registry metadata for a given `HLT-*` rule ID.

## Security and Permission Enforcement

When the MCP server executes a Tool, it will wrap the execution in the relevant **Permission Profile**:
- `read-only`: The server will only execute non-mutating commands (`audit`, `explain`).
- `docs-only`: The server will constrain context packs to documentation areas.
- `security-investigation`: The server will enforce aggressive stop conditions if secret sprawl or prompt injection is detected.

The MCP Server itself will run with the least privilege possible to prevent malicious context from taking over the host machine.
