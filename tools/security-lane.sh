#!/usr/bin/env bash
set -euo pipefail

strict="${JANKURAI_SECURITY_STRICT:-0}"

# One JSON object per line after the prefix; parsed by `jankurai security run`.
emit_step() {
  local label="$1"
  local tool="$2"
  local scmd="$3"
  local status="$4"
  local advisory_flag="$5"
  local ec="${6-}"
  if command -v python3 >/dev/null 2>&1; then
    _JS_LABEL="$label" _JS_TOOL="$tool" _JS_CMD="$scmd" _JS_STATUS="$status" \
      _JS_ADV="$advisory_flag" _JS_EC="$ec" python3 - <<'PY'
import json, os
label = os.environ["_JS_LABEL"]
tool = os.environ["_JS_TOOL"]
cmd = os.environ["_JS_CMD"]
status = os.environ["_JS_STATUS"]
adv = os.environ["_JS_ADV"] == "1"
ec_raw = os.environ.get("_JS_EC", "")
d = {
    "label": label,
    "tool": tool,
    "shell_command": cmd,
    "status": status,
    "advisory": adv,
}
if ec_raw != "":
    d["exit_code"] = int(ec_raw)
print("jankurai-security-step=" + json.dumps(d, ensure_ascii=False))
PY
  fi
}

run_required() {
  local tool="$1"
  local cmd="$2"
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "skip: $tool not installed; advisory outside strict mode"
    emit_step "$tool" "$tool" "$cmd" "skipped" "1" ""
    if [ "$strict" = "1" ]; then
      echo "missing required security tool: $tool" >&2
      exit 1
    fi
    return 0
  fi
  local err
  err="$(mktemp)"
  if bash -lc "$cmd" 2>"$err"; then
    emit_step "$tool" "$tool" "$cmd" "ran" "0" "0"
    rm -f "$err"
    return 0
  fi
  local code=$?
  emit_step "$tool" "$tool" "$cmd" "failed" "0" "$code"
  if [ -s "$err" ]; then
    cat "$err" >&2
  fi
  rm -f "$err"
  exit "$code"
}

run_advisory() {
  local tool="$1"
  local cmd="$2"
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "skip: $tool not installed; advisory outside strict mode"
    emit_step "$tool" "$tool" "$cmd" "skipped" "1" ""
    return 0
  fi
  local err
  err="$(mktemp)"
  if bash -lc "$cmd" 2>"$err"; then
    emit_step "$tool" "$tool" "$cmd" "ran" "1" "0"
    rm -f "$err"
    return 0
  fi
  local code=$?
  emit_step "$tool" "$tool" "$cmd" "failed" "1" "$code"
  if [ -s "$err" ]; then
    cat "$err" >&2
  fi
  rm -f "$err"
  exit "$code"
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
  run_required "${required_tool_names[$i]}" "${required_commands[$i]}"
done

for i in "${!advisory_tool_names[@]}"; do
  run_advisory "${advisory_tool_names[$i]}" "${advisory_commands[$i]}"
done
