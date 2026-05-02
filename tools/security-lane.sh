#!/usr/bin/env bash
set -euo pipefail

strict="${JANKURAI_SECURITY_STRICT:-0}"

run_or_skip() {
  local tool="$1"
  shift
  if command -v "$tool" >/dev/null 2>&1; then
    "$@"
    return 0
  fi

  echo "skip: $tool not installed; advisory outside strict mode"
  if [ "$strict" = "1" ]; then
    echo "missing required security tool: $tool" >&2
    exit 1
  fi
}

required_tool_names=(gitleaks cargo-audit npm)
required_commands=(
  "gitleaks detect --source . --redact --no-banner"
  "cargo audit"
  "npm audit --audit-level=high"
)

advisory_tool_names=(syft zizmor)
advisory_commands=(
  "syft . -o spdx-json=target/jankurai/sbom.spdx.json"
  "zizmor .github/workflows"
)

for i in "${!required_tool_names[@]}"; do
  tool="${required_tool_names[$i]}"
  command="${required_commands[$i]}"
  run_or_skip "$tool" bash -lc "$command"
done

for i in "${!advisory_tool_names[@]}"; do
  tool="${advisory_tool_names[$i]}"
  command="${advisory_commands[$i]}"
  run_or_skip "$tool" bash -lc "$command"
done
