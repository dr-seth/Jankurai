#!/usr/bin/env python3
"""Check humanlint version bindings across source, docs, and artifacts."""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "agent" / "standard-version.toml"
FORBIDDEN_LIVE_PAPER_NAMES = {"main.md", "main.tex", "main.pdf"}
IGNORED_NAME_CHECK_DIRS = {".git", "node_modules"}


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def parse_scalar_toml(path: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    for raw_line in read_text(path).splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#") or line.startswith("["):
            continue
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        value = value.strip()
        if value.startswith('"') and value.endswith('"'):
            value = value[1:-1]
        values[key.strip()] = value
    return values


def parse_artifacts(path: Path) -> list[dict[str, str]]:
    artifacts: list[dict[str, str]] = []
    current: dict[str, str] | None = None
    for raw_line in read_text(path).splitlines():
        line = raw_line.strip()
        if line == "[[artifact]]":
            current = {}
            artifacts.append(current)
            continue
        if current is None or not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        value = value.strip()
        if value.startswith('"') and value.endswith('"'):
            value = value[1:-1]
        current[key.strip()] = value
    return artifacts


def regex_value(path: Path, pattern: str, label: str) -> str:
    match = re.search(pattern, read_text(path), re.MULTILINE)
    if not match:
        raise AssertionError(f"{label}: value not found in {path.relative_to(ROOT)}")
    return match.group(1)


def assert_equal(label: str, actual: str, expected: str, errors: list[str]) -> None:
    if actual != expected:
        errors.append(f"{label}: expected {expected!r}, got {actual!r}")


def live_paper_name_violations() -> list[str]:
    violations: list[str] = []
    for path in ROOT.rglob("*"):
        if not path.is_file():
            continue
        rel = path.relative_to(ROOT)
        if any(part in IGNORED_NAME_CHECK_DIRS for part in rel.parts):
            continue
        if rel.as_posix() == "humanlint/__main__.py":
            continue
        if path.name.lower() in FORBIDDEN_LIVE_PAPER_NAMES:
            violations.append(rel.as_posix())
    return violations


def main() -> int:
    errors: list[str] = []
    manifest = parse_scalar_toml(MANIFEST)
    artifacts = parse_artifacts(MANIFEST)

    standard_version = manifest.get("standard_version", "")
    auditor_version = manifest.get("auditor_version", "")
    schema_version = manifest.get("schema_version", "")
    paper_edition = manifest.get("paper_edition", "")
    target_stack = manifest.get("target_stack", "")

    assert_equal("VERSION", read_text(ROOT / "VERSION").strip(), standard_version, errors)
    assert_equal(
        "pyproject.toml project.version",
        regex_value(ROOT / "pyproject.toml", r'^version = "([^"]+)"', "pyproject version"),
        standard_version,
        errors,
    )
    assert_equal(
        "humanlint/__init__.py __version__",
        regex_value(ROOT / "humanlint" / "__init__.py", r'__version__ = "([^"]+)"', "package version"),
        standard_version,
        errors,
    )

    cli_path = ROOT / "humanlint" / "cli.py"
    expected_constants = {
        "STANDARD_VERSION": standard_version,
        "AUDITOR_VERSION": auditor_version,
        "SCHEMA_VERSION": schema_version,
        "PAPER_EDITION": paper_edition,
        "TARGET_STACK_ID": target_stack,
    }
    for name, expected in expected_constants.items():
        actual = regex_value(cli_path, rf'^{name} = "([^"]+)"', name)
        assert_equal(f"humanlint/cli.py {name}", actual, expected, errors)

    standard_doc_version = regex_value(
        ROOT / "docs" / "agent-native-standard.md",
        r"Standard version: `([^`]+)`",
        "standard doc version",
    )
    assert_equal("docs/agent-native-standard.md version", standard_doc_version, standard_version, errors)

    brief_version = regex_value(
        ROOT / "agent" / "HUMANLINT_STANDARD.md",
        r"Standard version: `([^`]+)`",
        "agent brief version",
    )
    assert_equal("agent/HUMANLINT_STANDARD.md version", brief_version, standard_version, errors)

    paper_md = read_text(ROOT / "paper" / "humanlint.md")
    if f"Paper edition: `{paper_edition}`" not in paper_md:
        errors.append("paper/humanlint.md: paper edition does not match manifest")
    if f"Standard version: `{standard_version}`" not in paper_md:
        errors.append("paper/humanlint.md: standard version does not match manifest")

    required_artifacts = {
        "paper-source",
        "paper-render",
        "paper-agent-md",
        "coding-standard",
        "agent-standard-brief",
    }
    found_artifacts = {artifact.get("id", "") for artifact in artifacts}
    missing = sorted(required_artifacts - found_artifacts)
    if missing:
        errors.append("agent/standard-version.toml missing artifacts: " + ", ".join(missing))

    for artifact in artifacts:
        path = artifact.get("path")
        if path and not (ROOT / path).exists():
            errors.append(f"artifact {artifact.get('id', '<unknown>')}: missing path {path}")
        version_field = artifact.get("version_field")
        version = artifact.get("version")
        if version_field and version:
            expected = manifest.get(version_field, "")
            if expected and version != expected:
                errors.append(
                    f"artifact {artifact.get('id', '<unknown>')}: {version_field} expected {expected!r}, got {version!r}"
                )

    generated_zones = read_text(ROOT / "agent" / "generated-zones.toml")
    if 'path = "paper/humanlint.pdf"' not in generated_zones:
        errors.append("agent/generated-zones.toml: paper/humanlint.pdf is not registered")

    for violation in live_paper_name_violations():
        errors.append(
            f"{violation}: live paper artifacts must use `humanlint.*`; "
            "`main.*` paper names are not allowed in this repository"
        )

    if errors:
        for error in errors:
            print(f"version check failed: {error}", file=sys.stderr)
        return 1

    print(
        "versions ok: "
        f"standard={standard_version} auditor={auditor_version} "
        f"schema={schema_version} paper={paper_edition}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
