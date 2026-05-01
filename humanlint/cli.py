#!/usr/bin/env python3
"""Fast, dependency-free repo scorer for the humanlint rubric.

The scorer emits one JSON report plus one Markdown report from the same
analysis object. Optional ``--changed`` inputs narrow file-scoped inspection
while leaving repo-level routing checks in place.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable, Iterator

from .ux_readiness import ux_qa_status


STANDARD_VERSION = "0.2.0"
AUDITOR_VERSION = "0.2.0"
SCHEMA_VERSION = "1.0.0"
PAPER_EDITION = "2026.05-ed1"
TARGET_STACK_ID = "rust-ts-vite-react-postgres-bounded-python"
TARGET_STACK = "Rust core + TypeScript/React/Vite + PostgreSQL + generated contracts + bounded Python AI/data service"

WEIGHTS = {
    "Ownership and navigation surface": 14,
    "Contract and boundary integrity": 14,
    "Proof lanes and test routing": 14,
    "Security and supply-chain posture": 14,
    "Code shape and semantic surface": 12,
    "Data truth and workflow safety": 8,
    "Observability and repair evidence": 8,
    "Context economy and agent instructions": 8,
    "Python containment and polyglot hygiene": 4,
    "Build speed signals": 4,
}

CAPS = [
    ("no-root-agent-instructions", 75),
    ("no-one-command-setup-or-validation", 70),
    ("no-deterministic-fast-lane", 65),
    ("no-security-lane-on-high-risk-repo", 60),
    ("generated-contracts-or-public-api-drift-untested", 80),
    ("python-direct-product-truth-or-db-ownership", 72),
    ("no-secret-or-dependency-scanning-in-ci", 78),
    ("no-humanlint-audit-lane-in-ci", 82),
    ("non-optimal-product-language-found", 74),
    ("too-much-python-in-product-surface", 72),
    ("vibe-placeholders-in-product-code", 68),
    ("fallback-soup-in-product-code", 70),
    ("future-hostile-dead-language-in-product-code", 64),
    ("severe-duplication-in-product-code", 70),
    ("generated-zone-mutation-risk", 76),
    ("direct-db-access-from-wrong-layer", 66),
    ("missing-web-e2e-lane", 82),
    ("missing-rendered-ux-qa-lane", 84),
    ("missing-rust-property-or-integration-tests", 82),
    ("no-agent-friendly-exception-pattern", 76),
    ("missing-agent-readable-docs", 80),
]

EXCLUDED_DIRS = {
    ".git",
    ".hg",
    ".svn",
    ".idea",
    ".mypy_cache",
    ".pytest_cache",
    ".ruff_cache",
    ".tox",
    ".venv",
    "__pycache__",
    "build",
    "coverage",
    "dist",
    "node_modules",
    "target",
    "vendor",
    "venv",
    ".witness",
}

TEXT_BASENAMES = {
    "AGENTS.md",
    "CODEOWNERS",
    "Cargo.lock",
    "Cargo.toml",
    "Dockerfile",
    "Gemfile",
    "Gemfile.lock",
    "Justfile",
    "LICENSE",
    "Makefile",
    "Pipfile",
    "Pipfile.lock",
    "Procfile",
    "README",
    "README.md",
    "Taskfile.yaml",
    "Taskfile.yml",
    "build.gradle",
    "build.gradle.kts",
    "bunfig.toml",
    "clippy.toml",
    "go.mod",
    "go.sum",
    "justfile",
    "makefile",
    "package-lock.json",
    "package.json",
    "pnpm-lock.yaml",
    "poetry.lock",
    "pyproject.toml",
    "requirements.txt",
    "rust-toolchain.toml",
    "rustfmt.toml",
    "tsconfig.json",
    "uv.lock",
    "yarn.lock",
}

TEXT_EXTS = {
    ".c",
    ".cc",
    ".cfg",
    ".cjs",
    ".conf",
    ".cpp",
    ".cs",
    ".css",
    ".dart",
    ".d.ts",
    ".dockerfile",
    ".env",
    ".gitattributes",
    ".gitignore",
    ".go",
    ".gql",
    ".graphql",
    ".h",
    ".hh",
    ".hpp",
    ".htm",
    ".html",
    ".ini",
    ".java",
    ".js",
    ".json",
    ".jsx",
    ".kt",
    ".kts",
    ".ex",
    ".exs",
    ".lua",
    ".m",
    ".md",
    ".mk",
    ".mjs",
    ".mm",
    ".ps1",
    ".php",
    ".py",
    ".rb",
    ".rst",
    ".rs",
    ".sh",
    ".sql",
    ".swift",
    ".scala",
    ".tex",
    ".toml",
    ".ts",
    ".tsx",
    ".txt",
    ".yaml",
    ".yml",
}

CODE_EXTS = {
    ".c",
    ".cc",
    ".cpp",
    ".cs",
    ".dart",
    ".go",
    ".h",
    ".hh",
    ".hpp",
    ".java",
    ".js",
    ".jsx",
    ".kt",
    ".kts",
    ".ex",
    ".exs",
    ".lua",
    ".m",
    ".mm",
    ".py",
    ".php",
    ".rb",
    ".rs",
    ".sh",
    ".swift",
    ".scala",
    ".ts",
    ".tsx",
}

GENERATED_DIR_MARKERS = {
    "generated",
    "gen",
    "artifacts",
}

ALLOWED_PYTHON_ROOTS = {
    ".github",
    "benchmarks",
    "docs",
    "examples",
    "humanlint",
    "paper",
    "python/ai-service",
    "reference",
    "scripts",
    "tests",
    "tools",
}

STACK_PRODUCT_ALLOWED_SUFFIXES = {
    ".rs",
    ".ts",
    ".tsx",
    ".sql",
    ".proto",
    ".graphql",
    ".gql",
}

STACK_CONFIG_ALLOWED_SUFFIXES = {
    ".json",
    ".toml",
    ".yaml",
    ".yml",
    ".md",
    ".txt",
    ".sh",
    ".dockerfile",
}

NON_OPTIMAL_CODE_SUFFIXES = {
    ".c",
    ".cc",
    ".cpp",
    ".cs",
    ".dart",
    ".ex",
    ".exs",
    ".go",
    ".h",
    ".hh",
    ".hpp",
    ".java",
    ".js",
    ".jsx",
    ".kt",
    ".kts",
    ".lua",
    ".m",
    ".mm",
    ".php",
    ".rb",
    ".scala",
    ".swift",
}

ANALYSIS_EXCLUDED_PREFIXES = (
    "docs/",
    "humanlint/",
    "paper/",
    "reference/",
    "scripts/",
    "tests/",
    "tips/",
    "tools/",
)

SETUP_TARGETS = {
    "bootstrap",
    "build",
    "check",
    "ci",
    "dev",
    "init",
    "install",
    "setup",
    "test",
    "validate",
    "verify",
}

FAST_TARGETS = {
    "fast",
    "check",
    "test",
    "verify",
}

SECURITY_TARGETS = {
    "security",
    "scan",
    "safety",
}

FAST_COMMAND_MARKERS = {
    "cargo check",
    "cargo nextest",
    "nextest",
    "go test",
    "pytest",
    "pytest -q",
    "dotnet test",
    "vitest",
    "bun test",
    "pnpm test",
    "npm test",
    "yarn test",
    "python3 tools/humanlint.py",
}

SECURITY_TOOL_MARKERS = {
    "actionlint",
    "cargo-audit",
    "cargo-deny",
    "cargo-geiger",
    "cargo-vet",
    "dependency-review",
    "detect-secrets",
    "gitleaks",
    "grype",
    "pip-audit",
    "safety",
    "sbom",
    "slsa",
    "syft",
    "trivy",
    "zizmor",
}

CONTRACT_MARKERS = {
    "connectrpc",
    "openapi",
    "protobuf",
    "schemars",
    "specta",
    "sqlx",
    "ts-rs",
    "utoipa",
    "zod",
}

API_DRIFT_MARKERS = {
    "cargo-public-api",
    "cargo-semver-checks",
    "tsd",
    "api-extractor",
    "public api",
}

OBSERVABILITY_MARKERS = {
    "anyhow",
    "miette",
    "opentelemetry",
    "otel",
    "request_id",
    "tracing",
    "thiserror",
}

BUILD_SPEED_MARKERS = {
    "actions/cache",
    "bacon",
    "cargo build --timings",
    "cargo check",
    "cargo nextest",
    "cargo-chef",
    "cargo-binstall",
    "nx",
    "pnpm",
    "rolldown",
    "rust-cache",
    "sccache",
    "turbo",
    "vite",
    "vitest",
    "watchexec",
}

DB_MARKERS = {
    "diesel",
    "knex",
    "mysql",
    "postgres",
    "prisma",
    "psycopg",
    "query!(",
    "sequelize",
    "sqlite3",
    "sqlx",
    "typeorm",
}

DB_WRITE_MARKERS = {
    "delete ",
    "insert ",
    "merge ",
    "query!(",
    "sqlx::query",
    "update ",
}

DB_ALLOWED_PREFIXES = (
    "crates/adapters/",
    "db/",
    "migrations/",
)

DB_WRONG_LAYER_PREFIXES = (
    "apps/web/",
    "apps/api/",
    "crates/domain/",
    "crates/application/",
    "frontend/",
    "src/",
    "ui/",
)

DOMAIN_IO_MARKERS = {
    "fetch(",
    "file::",
    "open(",
    "os.",
    "print(",
    "println!",
    "read(",
    "requests.",
    "socket",
    "std::fs",
    "subprocess",
    "system.out",
    "write(",
}

TODO_STUB_PATTERNS = [
    re.compile(pattern, re.IGNORECASE)
    for pattern in (
        r"\bTODO\b",
        r"\bFIXME\b",
        r"\bHACK\b",
        r"\bXXX\b",
        r"\bstub\b",
        r"\bplaceholder\b",
        r"not\s+implemented",
        r"todo!\s*\(",
        r"unimplemented!\s*\(",
        r"unreachable!\s*\(",
        r"panic!\s*\(\s*\"(?:todo|not implemented)",
        r"throw\s+new\s+Error\s*\(\s*[\"'](?:todo|not implemented)",
    )
]

FALLBACK_SOUP_PATTERNS = [
    re.compile(pattern, re.IGNORECASE)
    for pattern in (
        r"\bfallback\b",
        r"\bbest\s*effort\b",
        r"\btry\s+again\b",
        r"\bretry\b",
        r"\bexcept\s+Exception\b",
        r"\bexcept\s*:",
        r"\bcatch\s*\([^)]*\)\s*\{\s*\}",
        r"\bcatch\s*\([^)]*(?:any|unknown|error|e)[^)]*\)",
        r"\bunwrap_or_default\s*\(",
        r"\bok\(\)\s*;",
        r"\bor_else\s*\(",
        r"\bcontinue\s*;\s*//\s*ignore",
        r"\breturn\s+null\b",
        r"\breturn\s+undefined\b",
    )
]

FUTURE_HOSTILE_DEAD_LANGUAGE_TERMS = (
    "cleanup later",
    "remove later",
    "best effort",
    "dead code",
    "deprecated",
    "depricated",
    "temporary",
    "workaround",
    "backcompat",
    "placeholder",
    "fallback",
    "obsolete",
    "legacy",
    "unused",
    "stale",
    "fixme",
    "dummy",
    "compat",
    "shim",
    "stub",
    "hack",
    "todo",
    "temp",
    "old",
)

FUTURE_HOSTILE_DEAD_LANGUAGE_PATTERN = re.compile(
    r"\b("
    + "|".join(
        re.escape(term).replace("\\ ", r"\s+")
        for term in sorted(FUTURE_HOSTILE_DEAD_LANGUAGE_TERMS, key=len, reverse=True)
    )
    + r")\b",
    re.IGNORECASE,
)

FUTURE_HOSTILE_ALLOWLIST_PREFIXES = (
    "docs/",
    "reference/",
    "vendor/",
)

FUTURE_HOSTILE_PRODUCT_COPY_PARTS = {
    "copy-deck",
    "copydeck",
    "i18n",
    "l10n",
    "locale",
    "locales",
    "marketing-copy",
    "messages",
    "product-copy",
    "productcopy",
    "translations",
}

FUTURE_HOSTILE_REASON = (
    "future-hostile/dead-language markers make product runtime behavior sound "
    "temporary, deprecated, abandoned, or safe to ignore instead of owned"
)

FUTURE_HOSTILE_AGENT_FIX = (
    "remove or rename the marker, implement the intended behavior, model a typed "
    "unsupported state, or move docs/generated/vendor/product-copy text into an allowlisted context"
)

HANDWRITTEN_API_PATTERNS = [
    re.compile(pattern, re.IGNORECASE)
    for pattern in (
        r"\bfetch\s*\(",
        r"\baxios\.",
        r"\bnew\s+XMLHttpRequest\b",
        r"\binterface\s+\w*(?:Dto|DTO|Request|Response|Payload)\b",
        r"\btype\s+\w*(?:Dto|DTO|Request|Response|Payload)\b",
        r"\bclass\s+\w*(?:Dto|DTO|Request|Response|Payload)\b",
    )
]

AGENT_EXCEPTION_MARKERS = {
    "common_fix",
    "common fixes",
    "common_fixes",
    "docs_url",
    "documentation",
    "error_code",
    "exception name",
    "purpose",
    "repair_hint",
}

WEAK_NAME_PARTS = {
    "common",
    "data",
    "helpers",
    "junk",
    "manager",
    "misc",
    "processor",
    "shared",
    "stuff",
    "thing",
    "tmp",
    "util",
    "utils",
}

DOC_REQUIRED_PATHS = {
    "AGENTS.md",
    "README.md",
    "docs/architecture.md",
    "docs/boundaries.md",
    "docs/testing.md",
    "docs/audit-rubric.md",
}

WEB_E2E_MARKERS = {
    "@playwright/test",
    "playwright",
    "cypress",
}

RUST_PROPERTY_TEST_MARKERS = {
    "proptest",
    "quickcheck",
    "rstest",
}

ROUTING_README_MARKERS = {
    "build",
    "flow",
    "layout",
    "map",
    "paper/",
    "reference/",
    "tools/",
    "validate",
    "workspace",
}

MEGAFILE_LOC = 500
VERY_LARGE_FILE_LOC = 1000
FUNCTION_LOC_SOFT = 80
FUNCTION_LOC_HARD = 140
PYTHON_PRODUCT_RATIO_SOFT = 0.15
PYTHON_PRODUCT_RATIO_HARD = 0.30
DUPLICATE_BLOCK_LINES = 8
DUPLICATE_BLOCK_LIMIT = 12
FUTURE_HOSTILE_FINDING_LIMIT = 64
MAX_CAPTURE_CHARS = 120_000


@dataclass
class FileInfo:
    rel_path: str
    abs_path: Path
    name: str
    suffix: str
    size: int
    line_count: int
    text: str

    @property
    def lower(self) -> str:
        return self.text.lower()

    @property
    def top_dir(self) -> str:
        parts = self.rel_path.split("/")
        return parts[0] if len(parts) > 1 else ""

    @property
    def is_generated(self) -> bool:
        parts = self.rel_path.split("/")
        return any(part in GENERATED_DIR_MARKERS or part.startswith("generated") for part in parts)

    @property
    def is_code(self) -> bool:
        if self.name in {"Makefile", "makefile", "Justfile", "justfile"}:
            return True
        return self.suffix in CODE_EXTS


@dataclass
class DimensionResult:
    name: str
    weight: int
    score: int
    weighted_points: float
    evidence: list[str] = field(default_factory=list)
    notes: list[str] = field(default_factory=list)


@dataclass
class Finding:
    severity: str
    category: str
    path: str
    problem: str
    agent_fix: str
    evidence: list[str]
    rule_id: str | None = None
    line: int | None = None
    matched_term: str | None = None
    reason: str | None = None

    def as_dict(self) -> dict:
        return {
            "severity": self.severity,
            "category": self.category,
            "path": self.path,
            "line": self.line,
            "matched_term": self.matched_term,
            "problem": self.problem,
            "reason": self.reason or self.problem,
            "agent_fix": self.agent_fix,
            "evidence": self.evidence,
            "rule_id": self.rule_id,
        }


@dataclass
class RepoContext:
    root: Path
    all_files: list[FileInfo]
    scope_files: list[FileInfo]
    scope_paths: list[str]
    scope_mode: str
    root_files: dict[str, FileInfo]

    @property
    def all_text_files(self) -> list[FileInfo]:
        return [f for f in self.all_files if f.text]

    @property
    def scope_text_files(self) -> list[FileInfo]:
        return [f for f in self.scope_files if f.text]

    @property
    def all_code_files(self) -> list[FileInfo]:
        return [f for f in self.all_files if f.is_code]

    @property
    def scope_code_files(self) -> list[FileInfo]:
        return [f for f in self.scope_files if f.is_code]

    @property
    def all_authored_files(self) -> list[FileInfo]:
        return [f for f in self.all_files if not f.is_generated]

    @property
    def scope_authored_files(self) -> list[FileInfo]:
        return [f for f in self.scope_files if not f.is_generated]

    @property
    def all_authored_code_files(self) -> list[FileInfo]:
        return [f for f in self.all_code_files if not f.is_generated]

    @property
    def scope_authored_code_files(self) -> list[FileInfo]:
        return [f for f in self.scope_code_files if not f.is_generated]


def clamp(value: int, lo: int = 0, hi: int = 100) -> int:
    return max(lo, min(hi, value))


def norm_relpath(root: Path, raw: str) -> str | None:
    candidate = Path(raw).expanduser()
    if not candidate.is_absolute():
        candidate = (root / candidate).resolve()
    else:
        candidate = candidate.resolve()
    try:
        rel = candidate.relative_to(root)
    except ValueError:
        return None
    if not candidate.exists():
        return None
    return rel.as_posix()


def path_matches_scope(rel_path: str, scopes: list[str]) -> bool:
    if not scopes:
        return True
    for scope in scopes:
        if rel_path == scope:
            return True
        if rel_path.startswith(scope + "/"):
            return True
        if scope.startswith(rel_path + "/"):
            return True
    return False


def is_text_candidate(rel_path: str) -> bool:
    name = rel_path.rsplit("/", 1)[-1]
    lower_name = name.lower()
    lower_path = rel_path.lower()
    if name in TEXT_BASENAMES or lower_name in {item.lower() for item in TEXT_BASENAMES}:
        return True
    if any(lower_path.endswith(ext) for ext in TEXT_EXTS):
        return True
    if lower_name in {"dockerfile", "makefile", "justfile"}:
        return True
    return False


def is_relevant_file(rel_path: str) -> bool:
    name = rel_path.rsplit("/", 1)[-1]
    lower_name = name.lower()
    if name in TEXT_BASENAMES or lower_name in {item.lower() for item in TEXT_BASENAMES}:
        return True
    if any(lower_name.endswith(ext) for ext in TEXT_EXTS):
        return True
    if lower_name in {"dockerfile", "makefile", "justfile"}:
        return True
    return False


def walk_relevant_files(root: Path) -> Iterator[Path]:
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [
            d for d in dirnames if d not in EXCLUDED_DIRS and d != ".git"
        ]
        dir_path = Path(dirpath)
        for filename in filenames:
            abs_path = dir_path / filename
            try:
                rel_path = abs_path.relative_to(root).as_posix()
            except ValueError:
                continue
            if is_relevant_file(rel_path):
                yield abs_path


def read_text_sample(path: Path) -> tuple[str, int]:
    line_count = 0
    captured: list[str] = []
    captured_chars = 0
    try:
        with path.open("r", encoding="utf-8", errors="replace") as handle:
            for line in handle:
                line_count += 1
                if captured_chars < MAX_CAPTURE_CHARS:
                    remaining = MAX_CAPTURE_CHARS - captured_chars
                    piece = line[:remaining]
                    captured.append(piece)
                    captured_chars += len(piece)
    except OSError:
        return "", 0
    return "".join(captured), line_count


def inventory_repo(root: Path) -> list[FileInfo]:
    files: list[FileInfo] = []
    for abs_path in walk_relevant_files(root):
        try:
            stat = abs_path.stat()
        except OSError:
            continue
        rel_path = abs_path.relative_to(root).as_posix()
        text = ""
        line_count = 0
        if is_text_candidate(rel_path):
            text, line_count = read_text_sample(abs_path)
        files.append(
            FileInfo(
                rel_path=rel_path,
                abs_path=abs_path,
                name=abs_path.name,
                suffix=abs_path.suffix.lower(),
                size=stat.st_size,
                line_count=line_count,
                text=text,
            )
        )
    files.sort(key=lambda item: item.rel_path)
    return files


def build_context(root: Path, all_files: list[FileInfo], changed: list[str]) -> RepoContext:
    scope_paths = [item for item in changed if item]
    if scope_paths:
        scoped = [f for f in all_files if path_matches_scope(f.rel_path, scope_paths)]
        scope_mode = "changed" if scoped else "full"
    else:
        scoped = list(all_files)
        scope_mode = "full"

    if scope_mode == "full":
        scoped = list(all_files)
        scope_paths = []

    root_files = {
        f.rel_path: f
        for f in all_files
        if "/" not in f.rel_path
    }
    return RepoContext(
        root=root,
        all_files=all_files,
        scope_files=scoped,
        scope_paths=scope_paths,
        scope_mode=scope_mode,
        root_files=root_files,
    )


def file_text(ctx: RepoContext, rel_path: str) -> str:
    file = ctx.root_files.get(rel_path)
    return file.text if file else ""


def any_file(ctx: RepoContext, predicate, scope: str = "all") -> FileInfo | None:
    files = ctx.all_files if scope == "all" else ctx.scope_files
    for file in files:
        if predicate(file):
            return file
    return None


def files_matching(ctx: RepoContext, predicate, scope: str = "all") -> list[FileInfo]:
    files = ctx.all_files if scope == "all" else ctx.scope_files
    return [file for file in files if predicate(file)]


def lower_join(files: Iterable[FileInfo]) -> str:
    return "\n".join(file.lower for file in files)


def is_product_surface_path(rel_path: str) -> bool:
    if "/" not in rel_path and rel_path in {"Justfile", "Makefile", "makefile", "justfile"}:
        return False
    if rel_path.startswith(ANALYSIS_EXCLUDED_PREFIXES):
        return False
    parts = rel_path.split("/")[:-1]
    return not any(part.startswith("_") for part in parts)


def product_files(ctx: RepoContext, scope: str = "scope") -> list[FileInfo]:
    files = ctx.scope_files if scope == "scope" else ctx.all_files
    return [file for file in files if is_product_surface_path(file.rel_path)]


def product_code_files(ctx: RepoContext, scope: str = "scope") -> list[FileInfo]:
    return [file for file in product_files(ctx, scope=scope) if file.is_code]


def has_prefix(ctx: RepoContext, prefix: str, scope: str = "all") -> bool:
    files = ctx.all_files if scope == "all" else ctx.scope_files
    for file in files:
        if file.rel_path == prefix or file.rel_path.startswith(prefix.rstrip("/") + "/"):
            return True
    return False


def count_large_files(files: Iterable[FileInfo], threshold: int) -> int:
    return sum(1 for file in files if file.is_code and not file.is_generated and file.line_count > threshold)


def max_loc(files: Iterable[FileInfo]) -> int:
    values = [file.line_count for file in files if file.is_code and not file.is_generated]
    return max(values, default=0)


def starts_with_any(rel_path: str, prefixes: Iterable[str]) -> bool:
    return any(rel_path == prefix.rstrip("/") or rel_path.startswith(prefix.rstrip("/") + "/") for prefix in prefixes)


def is_runtime_stack_surface(file: FileInfo) -> bool:
    if file.is_generated:
        return False
    if "/" not in file.rel_path and file.name in {"Justfile", "Makefile", "makefile", "justfile"}:
        return False
    if starts_with_any(file.rel_path, ("contracts/", "db/", "migrations/", "ops/", ".github/")):
        return False
    return is_product_surface_path(file.rel_path)


def runtime_code_files(ctx: RepoContext, scope: str = "scope") -> list[FileInfo]:
    files = ctx.scope_files if scope == "scope" else ctx.all_files
    return [file for file in files if file.is_code and is_runtime_stack_surface(file)]


def product_line_total(files: Iterable[FileInfo]) -> int:
    return sum(file.line_count for file in files if file.line_count > 0)


def non_optimal_language_hits(ctx: RepoContext) -> list[FileInfo]:
    hits: list[FileInfo] = []
    for file in runtime_code_files(ctx, scope="all"):
        if file.suffix in NON_OPTIMAL_CODE_SUFFIXES:
            hits.append(file)
        elif file.suffix == ".py" and not file.rel_path.startswith("python/ai-service/"):
            hits.append(file)
    return hits


def python_product_ratio(ctx: RepoContext) -> tuple[float, int, int]:
    files = runtime_code_files(ctx, scope="all")
    total = product_line_total(files)
    python_lines = sum(file.line_count for file in files if file.suffix == ".py")
    if total <= 0:
        return 0.0, python_lines, total
    return python_lines / total, python_lines, total


def pattern_hits(files: Iterable[FileInfo], patterns: list[re.Pattern], limit: int = 20) -> list[dict]:
    hits: list[dict] = []
    for file in files:
        if not file.text:
            continue
        for line_no, line in enumerate(file.text.splitlines(), start=1):
            if any(pattern.search(line) for pattern in patterns):
                hits.append(
                    {
                        "path": file.rel_path,
                        "line": line_no,
                        "text": line.strip()[:160],
                    }
                )
                if len(hits) >= limit:
                    return hits
    return hits


def todo_stub_hits(ctx: RepoContext) -> list[dict]:
    return pattern_hits(runtime_code_files(ctx), TODO_STUB_PATTERNS)


def fallback_soup_hits(ctx: RepoContext) -> list[dict]:
    hits = pattern_hits(runtime_code_files(ctx), FALLBACK_SOUP_PATTERNS)
    # A single explicit fallback may be legitimate. Multiple files or repeated hits become a repair signal.
    if len(hits) <= 1:
        return []
    return hits


def is_future_hostile_allowlisted_context(file: FileInfo) -> bool:
    if file.is_generated:
        return True
    if has_any(file.lower[:4000], {"@generated", "auto-generated", "do not edit", "codegen"}):
        return True
    if starts_with_any(file.rel_path, FUTURE_HOSTILE_ALLOWLIST_PREFIXES):
        return True

    parts = {part.lower().replace("_", "-") for part in Path(file.rel_path).parts}
    if parts.intersection(FUTURE_HOSTILE_PRODUCT_COPY_PARTS):
        return True

    stem = Path(file.name).stem.lower().replace("_", "-")
    return stem in FUTURE_HOSTILE_PRODUCT_COPY_PARTS


def future_hostile_dead_language_hits(ctx: RepoContext) -> list[dict]:
    hits: list[dict] = []
    for file in runtime_code_files(ctx):
        if not file.text or is_future_hostile_allowlisted_context(file):
            continue
        for line_no, line in enumerate(file.text.splitlines(), start=1):
            match = FUTURE_HOSTILE_DEAD_LANGUAGE_PATTERN.search(line)
            if not match:
                continue
            matched_term = re.sub(r"\s+", " ", match.group(1).strip().lower())
            hits.append(
                {
                    "path": file.rel_path,
                    "line": line_no,
                    "matched_term": matched_term,
                    "reason": FUTURE_HOSTILE_REASON,
                    "agent_fix": FUTURE_HOSTILE_AGENT_FIX,
                    "text": line.strip()[:160],
                }
            )
            if len(hits) >= FUTURE_HOSTILE_FINDING_LIMIT:
                return hits
    return hits


def handwritten_api_hits(ctx: RepoContext) -> list[dict]:
    candidates = [
        file
        for file in product_code_files(ctx)
        if file.suffix in {".ts", ".tsx"}
        and not file.is_generated
        and starts_with_any(file.rel_path, ("apps/web/", "frontend/", "ui/", "src/"))
    ]
    return pattern_hits(candidates, HANDWRITTEN_API_PATTERNS)


def generated_zone_issues(ctx: RepoContext) -> list[dict]:
    generated_files = [file for file in ctx.all_files if file.is_generated and file.text and file.is_code]
    if not generated_files:
        return []

    issues: list[dict] = []
    has_zone_manifest = any(file.rel_path in {"generated-zones.toml", "agent/generated-zones.toml"} for file in ctx.all_files)
    if not has_zone_manifest:
        first = generated_files[0]
        issues.append(
            {
                "path": first.rel_path,
                "line": 1,
                "text": "generated code exists without `agent/generated-zones.toml` ownership rules",
            }
        )

    for file in generated_files:
        lower = file.lower[:4000]
        if not has_any(lower, {"generated", "auto-generated", "@generated", "do not edit", "codegen"}):
            issues.append(
                {
                    "path": file.rel_path,
                    "line": 1,
                    "text": "generated file lacks a clear generated/do-not-edit marker",
                }
            )
        if pattern_hits([file], TODO_STUB_PATTERNS, limit=1):
            issues.append(
                {
                    "path": file.rel_path,
                    "line": 1,
                    "text": "generated file contains TODO/stub markers",
                }
            )
        if len(issues) >= 20:
            break
    return issues


def wrong_layer_db_hits(ctx: RepoContext) -> list[dict]:
    hits: list[dict] = []
    files = product_files(ctx)
    for file in files:
        if not file.text or file.is_generated:
            continue
        if not (file.is_code or file.suffix == ".sql"):
            continue
        if starts_with_any(file.rel_path, DB_ALLOWED_PREFIXES):
            continue
        if starts_with_any(file.rel_path, DB_WRONG_LAYER_PREFIXES) and has_any(file.text, DB_MARKERS | DB_WRITE_MARKERS):
            first_line = 1
            for line_no, line in enumerate(file.text.splitlines(), start=1):
                if has_any(line, DB_MARKERS | DB_WRITE_MARKERS):
                    first_line = line_no
                    break
            hits.append({"path": file.rel_path, "line": first_line, "text": "DB marker in non-adapter layer"})
        elif file.suffix == ".py" and has_any(file.text, {"psycopg", "sqlalchemy", "postgres", "sqlite3", "mysql"}):
            hits.append({"path": file.rel_path, "line": 1, "text": "Python DB client marker"})
        if len(hits) >= 20:
            break
    return hits


def weak_name_hits(ctx: RepoContext) -> list[dict]:
    hits: list[dict] = []
    for file in runtime_code_files(ctx):
        parts = [part.lower() for part in Path(file.rel_path).parts]
        stem = Path(file.name).stem.lower()
        weak_parts = [part for part in parts[:-1] if part in WEAK_NAME_PARTS]
        weak_file = stem in WEAK_NAME_PARTS or stem.endswith("_utils") or stem.endswith("-utils")
        if weak_parts or weak_file:
            hits.append(
                {
                    "path": file.rel_path,
                    "line": 1,
                    "text": f"weak name token(s): {', '.join(sorted(set(weak_parts + ([stem] if weak_file else []))))}",
                }
            )
        if len(hits) >= 20:
            break
    return hits


def normalized_code_lines(file: FileInfo) -> list[tuple[int, str]]:
    lines: list[tuple[int, str]] = []
    for line_no, raw in enumerate(file.text.splitlines(), start=1):
        line = raw.strip()
        if not line:
            continue
        if line.startswith(("//", "#", "/*", "*", "--")):
            continue
        if re.match(r"^(use|import|export\s+\*)\b", line):
            continue
        if line in {"{", "}", "};", ");", "};"}:
            continue
        normalized = re.sub(r'"[^"]*"|\'[^\']*\'|`[^`]*`', '"S"', line)
        normalized = re.sub(r"\b\d+(?:\.\d+)?\b", "N", normalized)
        normalized = re.sub(r"\s+", " ", normalized)
        if len(normalized) < 12:
            continue
        lines.append((line_no, normalized))
    return lines


def duplicate_blocks(ctx: RepoContext) -> list[dict]:
    seen: dict[str, tuple[str, int]] = {}
    duplicates: list[dict] = []
    for file in runtime_code_files(ctx):
        if not file.text or file.line_count < DUPLICATE_BLOCK_LINES * 2:
            continue
        normalized = normalized_code_lines(file)
        for idx in range(0, max(0, len(normalized) - DUPLICATE_BLOCK_LINES + 1)):
            window = normalized[idx : idx + DUPLICATE_BLOCK_LINES]
            body = "\n".join(line for _, line in window)
            digest = hashlib.sha1(body.encode("utf-8")).hexdigest()
            current = (file.rel_path, window[0][0])
            previous = seen.get(digest)
            if previous and previous[0] != file.rel_path:
                duplicates.append(
                    {
                        "path": file.rel_path,
                        "line": window[0][0],
                        "text": f"duplicate block also appears at {previous[0]}:{previous[1]}",
                    }
                )
                if len(duplicates) >= DUPLICATE_BLOCK_LIMIT:
                    return duplicates
            else:
                seen[digest] = current
    return duplicates


def brace_delta(line: str) -> int:
    stripped = re.sub(r'"[^"]*"|\'[^\']*\'|`[^`]*`', "", line)
    return stripped.count("{") - stripped.count("}")


def large_function_hits(ctx: RepoContext) -> list[dict]:
    hits: list[dict] = []
    for file in runtime_code_files(ctx):
        if file.suffix == ".py":
            hits.extend(large_python_functions(file))
        elif file.suffix in {".rs", ".ts", ".tsx", ".js", ".jsx"}:
            hits.extend(large_brace_functions(file))
        if len(hits) >= 20:
            break
    return hits[:20]


def large_python_functions(file: FileInfo) -> list[dict]:
    hits: list[dict] = []
    active: tuple[int, int, str] | None = None
    for line_no, line in enumerate(file.text.splitlines(), start=1):
        if re.match(r"^\s*(def|async\s+def)\s+\w+", line):
            indent = len(line) - len(line.lstrip(" "))
            if active:
                start, _, name = active
                span = line_no - start
                if span > FUNCTION_LOC_SOFT:
                    hits.append({"path": file.rel_path, "line": start, "text": f"{name} is about {span} LOC"})
            active = (line_no, indent, line.strip().split("(", 1)[0])
            continue
        if active and line.strip() and not line.lstrip().startswith(("#", "@")):
            start, indent, name = active
            current_indent = len(line) - len(line.lstrip(" "))
            if current_indent <= indent and line_no > start:
                span = line_no - start
                if span > FUNCTION_LOC_SOFT:
                    hits.append({"path": file.rel_path, "line": start, "text": f"{name} is about {span} LOC"})
                active = None
    if active:
        start, _, name = active
        span = len(file.text.splitlines()) - start + 1
        if span > FUNCTION_LOC_SOFT:
            hits.append({"path": file.rel_path, "line": start, "text": f"{name} is about {span} LOC"})
    return hits


def large_brace_functions(file: FileInfo) -> list[dict]:
    hits: list[dict] = []
    start_line = 0
    name = ""
    depth = 0
    body_started = False
    start_pattern = re.compile(
        r"\b(fn|function)\s+([A-Za-z_][A-Za-z0-9_]*)\b|"
        r"\b(?:const|let|var)\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(?:async\s*)?(?:\([^)]*\)|[A-Za-z_][A-Za-z0-9_]*)\s*=>"
    )
    lines = file.text.splitlines()
    for line_no, line in enumerate(lines, start=1):
        if start_line == 0:
            match = start_pattern.search(line)
            if not match:
                continue
            start_line = line_no
            name = next((group for group in match.groups()[1:] if group), "function")
            depth = 0
            body_started = False

        delta = brace_delta(line)
        if "{" in line:
            body_started = True
        depth += delta
        if body_started and depth <= 0 and line_no > start_line:
            span = line_no - start_line + 1
            if span > FUNCTION_LOC_SOFT:
                hits.append({"path": file.rel_path, "line": start_line, "text": f"{name} is about {span} LOC"})
            start_line = 0
            name = ""
            depth = 0
            body_started = False

    if start_line:
        span = len(lines) - start_line + 1
        if span > FUNCTION_LOC_SOFT:
            hits.append({"path": file.rel_path, "line": start_line, "text": f"{name} is about {span} LOC"})
    return hits


def has_web_surface(ctx: RepoContext) -> bool:
    if any(has_prefix(ctx, prefix) for prefix in ("apps/web", "web", "frontend", "ui", "packages/web", "packages/ui")):
        return True
    package_text = lower_join(file for file in ctx.all_files if file.name == "package.json")
    if has_any(package_text, {"react", "vite", "next", "storybook"}):
        return True
    return any(
        file.suffix in {".tsx", ".ts"} and starts_with_any(file.rel_path, ("frontend/", "ui/", "packages/web/", "packages/ui/"))
        for file in ctx.all_files
    )


def has_rust_surface(ctx: RepoContext) -> bool:
    return any(file.suffix == ".rs" and is_runtime_stack_surface(file) for file in ctx.all_files)


def has_playwright_e2e(ctx: RepoContext) -> bool:
    if not has_web_surface(ctx):
        return True
    surface_text, names = concat_command_surface(ctx)
    if has_any(surface_text, WEB_E2E_MARKERS | {"e2e"}) or any("e2e" in name for name in names):
        return True
    return any(
        file.text
        and (has_any(file.text, WEB_E2E_MARKERS) or starts_with_any(file.rel_path, ("apps/web/e2e/", "e2e/", "tests/e2e/")))
        for file in ctx.all_files
    )


def has_rust_property_tests(ctx: RepoContext) -> bool:
    if not has_rust_surface(ctx):
        return True
    return any(file.text and has_any(file.text, RUST_PROPERTY_TEST_MARKERS) for file in ctx.all_files)


def has_rust_integration_tests(ctx: RepoContext) -> bool:
    if not has_rust_surface(ctx):
        return True
    return any(
        file.suffix == ".rs"
        and (
            "/tests/" in file.rel_path
            or file.rel_path.startswith("tests/")
            or re.search(r"#\s*\[\s*tokio::test\s*\]|#\s*\[\s*test\s*\]", file.text)
        )
        for file in ctx.all_files
    )


def has_humanlint_audit_ci_lane(ctx: RepoContext) -> bool:
    workflow_text = lower_join(file for file in ctx.all_files if file.rel_path.startswith(".github/workflows/"))
    return has_any(workflow_text, {"humanlint.py", "repo-score", "agent_fix_queue", "audit-rubric", "humanlint audit"})


def missing_core_docs(ctx: RepoContext) -> list[str]:
    present = {file.rel_path for file in ctx.all_files}
    required = ["AGENTS.md", "README.md"]
    missing = [path for path in required if path not in present]
    if not any(path in present for path in ("docs/architecture.md", "docs/boundaries.md")):
        missing.append("docs/architecture.md or docs/boundaries.md")
    if has_web_surface(ctx) or has_rust_surface(ctx):
        if "docs/testing.md" not in present and not any(file.rel_path.endswith("/testing.md") for file in ctx.all_files):
            missing.append("docs/testing.md")
    return missing


def has_agent_friendly_exceptions(ctx: RepoContext) -> bool:
    files = runtime_code_files(ctx, scope="all")
    for file in files:
        if not file.text:
            continue
        lower = file.lower
        error_shape = has_any(
            lower,
            {
                "derive(error",
                "enum error",
                "impl std::error::error",
                "extends error",
                "class ",
                "thiserror",
                "exception",
            },
        )
        marker_count = sum(1 for marker in AGENT_EXCEPTION_MARKERS if marker in lower)
        if error_shape and marker_count >= 3:
            return True
    return False


def extract_package_scripts(file: FileInfo) -> dict[str, str]:
    if file.name != "package.json" or not file.text:
        return {}
    try:
        data = json.loads(file.text)
    except json.JSONDecodeError:
        return {}
    scripts = data.get("scripts", {})
    if not isinstance(scripts, dict):
        return {}
    return {str(key): str(value) for key, value in scripts.items() if isinstance(key, str)}


def extract_named_targets(file: FileInfo) -> dict[str, str]:
    if not file.text:
        return {}
    name = file.name.lower()
    if name == "package.json":
        return extract_package_scripts(file)
    targets: dict[str, str] = {}
    if name in {"justfile", "makefile"}:
        pattern = re.compile(r"^([A-Za-z0-9_.-]+)\s*:\s*(?:#.*)?$")
        for line in file.text.splitlines():
            if line.startswith("\t") or line.startswith(" "):
                continue
            match = pattern.match(line)
            if not match:
                continue
            target = match.group(1)
            if target.startswith("."):
                continue
            targets[target] = ""
        return targets
    if name in {"taskfile.yml", "taskfile.yaml"}:
        pattern = re.compile(r"^\s{2,4}([A-Za-z0-9_.-]+):\s*(?:#.*)?$")
        for line in file.text.splitlines():
            match = pattern.match(line)
            if not match:
                continue
            target = match.group(1)
            if target in {"env", "vars", "includes", "tasks", "version"}:
                continue
            targets[target] = ""
        return targets
    return {}


def concat_command_surface(ctx: RepoContext) -> tuple[str, set[str]]:
    files = []
    names: set[str] = set()
    for file in ctx.all_files:
        if file.name.lower() in {"justfile", "makefile", "package.json", "taskfile.yaml", "taskfile.yml"} or file.rel_path.startswith(".github/workflows"):
            files.append(file.text)
            names.update(extract_named_targets(file).keys())
    return ("\n".join(text for text in files if text), names)


def has_any(text: str, needles: Iterable[str]) -> bool:
    lower = text.lower()
    return any(needle in lower for needle in needles)


def list_matching_paths(files: Iterable[FileInfo], prefix: str) -> list[str]:
    return [file.rel_path for file in files if file.rel_path == prefix or file.rel_path.startswith(prefix.rstrip("/") + "/")]


def find_text_files_with_markers(files: Iterable[FileInfo], markers: set[str]) -> list[FileInfo]:
    matches: list[FileInfo] = []
    for file in files:
        if not file.text:
            continue
        if has_any(file.text, markers):
            matches.append(file)
    return matches


def root_readme_routes(readme: FileInfo | None) -> bool:
    if not readme or not readme.text:
        return False
    lower = readme.lower
    return any(marker in lower for marker in ROUTING_README_MARKERS)


def is_allowed_python_path(rel_path: str) -> bool:
    normalized = rel_path.replace("\\", "/")
    for prefix in ALLOWED_PYTHON_ROOTS:
        if normalized == prefix or normalized.startswith(prefix.rstrip("/") + "/"):
            return True
    return False


def is_high_risk_repo(ctx: RepoContext) -> bool:
    return bool(product_code_files(ctx, scope="all") or any(file.name in {"package.json", "pyproject.toml", "Cargo.toml", "go.mod"} for file in ctx.all_files))


def has_contract_surface(ctx: RepoContext) -> bool:
    if has_prefix(ctx, "contracts"):
        return True
    for file in product_files(ctx, scope="all"):
        if not file.text:
            continue
        if file.suffix in {".proto", ".graphql", ".gql"}:
            return True
        if not (file.is_code or file.suffix in {".json", ".toml", ".yaml", ".yml"}):
            continue
        if has_any(file.text, CONTRACT_MARKERS):
            return True
    return False


def has_polyglot_boundary(ctx: RepoContext) -> bool:
    return (
        has_prefix(ctx, "apps/web")
        or has_prefix(ctx, "apps/api")
        or has_prefix(ctx, "crates/domain")
        or has_prefix(ctx, "crates/application")
        or has_prefix(ctx, "crates/adapters")
    )


def has_generated_contracts(ctx: RepoContext) -> bool:
    for file in product_files(ctx, scope="all"):
        if not file.text:
            continue
        if not file.is_generated and "generated" not in file.rel_path:
            continue
        if has_any(file.text, CONTRACT_MARKERS | {"generated", "do not edit", "auto-generated"}):
            return True
    return False


def has_api_drift_checks(ctx: RepoContext) -> bool:
    surface_text, names = concat_command_surface(ctx)
    if names.intersection(API_DRIFT_MARKERS):
        return True
    return has_any(surface_text, API_DRIFT_MARKERS)


def has_security_lane(ctx: RepoContext) -> bool:
    surface_text, names = concat_command_surface(ctx)
    if names.intersection(SECURITY_TARGETS):
        return True
    return has_any(surface_text, SECURITY_TOOL_MARKERS | {"security", "scan"})


def has_fast_lane(ctx: RepoContext) -> bool:
    surface_text, names = concat_command_surface(ctx)
    if names.intersection(FAST_TARGETS):
        return has_any(surface_text, FAST_COMMAND_MARKERS)
    return has_any(surface_text, FAST_COMMAND_MARKERS)


def has_one_command_setup(ctx: RepoContext) -> bool:
    surface_text, names = concat_command_surface(ctx)
    if names.intersection(SETUP_TARGETS):
        return True
    if has_any(surface_text, {"setup", "bootstrap", "install", "init", "dev", "verify", "validate"}):
        return True
    return False


def has_secret_or_dependency_scans(ctx: RepoContext) -> bool:
    surface_text, names = concat_command_surface(ctx)
    if names.intersection(SECURITY_TARGETS):
        return has_any(surface_text, SECURITY_TOOL_MARKERS)
    return has_any(surface_text, SECURITY_TOOL_MARKERS)


def make_dimension(
    name: str,
    score: int,
    evidence: list[str],
    notes: list[str] | None = None,
) -> DimensionResult:
    weight = WEIGHTS[name]
    return DimensionResult(
        name=name,
        weight=weight,
        score=clamp(score),
        weighted_points=round(weight * clamp(score) / 100.0, 2),
        evidence=evidence,
        notes=notes or [],
    )


def ownership_dimension(ctx: RepoContext) -> DimensionResult:
    score = 35
    evidence: list[str] = []
    notes: list[str] = []

    root_agents = ctx.root_files.get("AGENTS.md")
    if root_agents:
        score += 18
        evidence.append("root `AGENTS.md` present")
        if root_agents.line_count <= 120 and root_agents.size <= 4096:
            score += 8
        else:
            notes.append("root `AGENTS.md` is longer than a terse router should be")
    else:
        notes.append("no root `AGENTS.md`")

    if any(file.name == "CODEOWNERS" for file in ctx.all_files):
        score += 10
        evidence.append("`CODEOWNERS` present")

    if any(file.rel_path in {"agent-map.json", "agent/owner-map.json"} for file in ctx.all_files):
        score += 10
        evidence.append("owner map present")

    if any(file.rel_path in {"test-map.json", "agent/test-map.json", "proof-lanes.toml"} for file in ctx.all_files):
        score += 8
        evidence.append("test/proof routing map present")

    local_agents = [file for file in ctx.all_files if file.name == "AGENTS.md" and file.rel_path != "AGENTS.md"]
    if local_agents:
        score += min(8, 2 * len(local_agents))
        evidence.append(f"{len(local_agents)} local `AGENTS.md` file(s)")

    readme = ctx.root_files.get("README.md")
    if root_readme_routes(readme):
        score += 6
        evidence.append("root `README.md` routes to workspace layout")

    authored = product_code_files(ctx, scope="all")
    if authored:
        largest = max_loc(authored)
        if largest > MEGAFILE_LOC:
            score -= 8
            evidence.append(f"authored code file exceeds {MEGAFILE_LOC} LOC")
        if largest > VERY_LARGE_FILE_LOC:
            score -= 8
            evidence.append(f"authored code file exceeds {VERY_LARGE_FILE_LOC} LOC")
        if count_large_files(authored, MEGAFILE_LOC) > 2:
            score -= 4
            notes.append("multiple large code files")
    else:
        score -= 4
        notes.append("no authored code files found")

    return make_dimension("Ownership and navigation surface", score, evidence, notes)


def contract_dimension(ctx: RepoContext) -> DimensionResult:
    score = 35
    evidence: list[str] = []
    notes: list[str] = []

    has_contracts = has_contract_surface(ctx)
    if has_contracts:
        score += 15
        evidence.append("contract surface found")

    generated_contracts = has_generated_contracts(ctx)
    if generated_contracts:
        score += 15
        evidence.append("generated contract artifacts found")
    if generated_zone_issues(ctx):
        score -= 12
        evidence.append("generated zone mutation risk detected")

    if has_polyglot_boundary(ctx):
        score += 10
        evidence.append("polyglot boundary layout present")

    if has_api_drift_checks(ctx):
        score += 10
        evidence.append("public API drift checks found")

    scope_product = product_files(ctx)

    if any(file.rel_path.endswith(".ts") or file.rel_path.endswith(".tsx") for file in scope_product):
        tsconfig = ctx.root_files.get("tsconfig.json")
        if tsconfig and "strict" in tsconfig.lower:
            score += 10
            evidence.append("TypeScript strict mode hinted by `tsconfig.json`")

    if any(file.rel_path.endswith(".rs") for file in scope_product):
        if any(has_any(file.text, {"serde", "thiserror", "anyhow"}) for file in scope_product if file.text):
            score += 8
            evidence.append("Rust typed boundary helpers found")

    frontend_api = any(
        file.text and file.rel_path.startswith(("apps/web", "web", "frontend", "ui", "src"))
        and has_any(file.text, {"fetch(", "axios", "graphql", "request("})
        for file in scope_product
    )
    if frontend_api and not generated_contracts:
        score -= 15
        evidence.append("frontend appears to hand-write API access")
        notes.append("frontend API surface is not clearly generated")

    handwritten_api = handwritten_api_hits(ctx)
    if handwritten_api:
        score -= 15
        evidence.append(f"handwritten web DTO/API marker: {handwritten_api[0]['path']}:{handwritten_api[0]['line']}")
        notes.append("web API types should be generated from contracts")

    wrong_layer_sql = any(
        file.text
        and file.rel_path.startswith(("apps/web", "crates/domain", "frontend", "ui", "src"))
        and has_any(file.text, {"select ", "insert ", "update ", "delete ", "sqlx", "diesel", "psycopg", "sqlite3"})
        for file in scope_product
    )
    if wrong_layer_sql:
        score -= 10
        evidence.append("DB access found in a likely wrong layer")

    return make_dimension("Contract and boundary integrity", score, evidence, notes)


def proof_lanes_dimension(ctx: RepoContext) -> DimensionResult:
    score = 20
    evidence: list[str] = []
    notes: list[str] = []

    if has_one_command_setup(ctx):
        score += 15
        evidence.append("one-command setup/validation lane found")
    else:
        notes.append("no one-command setup/validation lane")

    if has_fast_lane(ctx):
        score += 15
        evidence.append("deterministic fast lane found")
    else:
        notes.append("no deterministic fast lane")

    surface_text, _ = concat_command_surface(ctx)
    if has_any(surface_text, {"cargo test", "cargo nextest", "go test", "pytest", "vitest", "dotnet test", "npm test"}):
        score += 10
        evidence.append("test runner present in automation surface")

    if has_prefix(ctx, ".github/workflows"):
        score += 8
        evidence.append("GitHub workflow files present")

    if any(file.rel_path in {"test-map.json", "agent/test-map.json", "proof-lanes.toml"} for file in ctx.all_files):
        score += 8
        evidence.append("test/proof routing map present")

    if has_any(surface_text, {"fast", "check", "test", "verify", "ci"}) and has_any(surface_text, {"cargo check", "nextest", "vitest", "pytest", "go test"}):
        score += 8
        evidence.append("lane names match fast validation commands")

    if has_humanlint_audit_ci_lane(ctx):
        score += 8
        evidence.append("humanlint audit lane found in CI")
    elif is_high_risk_repo(ctx):
        score -= 8
        notes.append("no humanlint audit lane in CI")

    if has_playwright_e2e(ctx):
        score += 8
        evidence.append("web e2e lane present or no web surface")
    else:
        score -= 10
        notes.append("web surface lacks Playwright/Cypress e2e lane")

    ux_qa = ux_qa_status(ctx, has_web_surface(ctx))
    if ux_qa.has_rendered_ux_lane:
        score += 6
        evidence.append("rendered UX QA lane present or no web surface")
    else:
        score -= 6
        notes.append("web surface lacks layered rendered UX QA")

    if ux_qa.geometry_runtime:
        score += 4
        evidence.append("DOM geometry UX QA runtime found")

    if has_rust_property_tests(ctx) and has_rust_integration_tests(ctx):
        score += 8
        evidence.append("Rust property/integration tests present or no Rust surface")
    else:
        score -= 10
        notes.append("Rust surface lacks property or integration tests")

    if not has_any(surface_text, {"cargo", "pytest", "go test", "vitest", "dotnet test", "npm test"}):
        score -= 10
        notes.append("no obvious test automation commands")

    return make_dimension("Proof lanes and test routing", score, evidence, notes)


def security_dimension(ctx: RepoContext) -> DimensionResult:
    score = 20
    evidence: list[str] = []
    notes: list[str] = []

    root_files = " ".join(file.name for file in ctx.all_files)
    if any(name in root_files for name in {"Cargo.lock", "package-lock.json", "pnpm-lock.yaml", "yarn.lock", "poetry.lock", "uv.lock", "Gemfile.lock"}):
        score += 12
        evidence.append("lockfile present")

    surface_text, _ = concat_command_surface(ctx)
    if has_any(surface_text, {"gitleaks", "detect-secrets", "secret", "audit", "deny", "dependency-review"}):
        score += 12
        evidence.append("secret or dependency scan tooling found")

    if has_any(surface_text, {"syft", "grype", "slsa", "sbom", "cosign"}):
        score += 8
        evidence.append("provenance/SBOM tooling found")

    if has_any(surface_text, {"actionlint", "zizmor"}):
        score += 8
        evidence.append("workflow linting tooling found")

    if has_security_lane(ctx):
        score += 8
        evidence.append("security lane present")
    else:
        notes.append("no explicit security lane found")

    if has_humanlint_audit_ci_lane(ctx):
        score += 6
        evidence.append("agent-readiness audit gate found in CI")
    else:
        score -= 6
        notes.append("CI does not run the humanlint audit")

    if any(file.text and has_any(file.text, {"unsafe", "unsafe "}) for file in product_code_files(ctx) if file.name.endswith(".rs")):
        score += 4
        evidence.append("unsafe usage appears to be tracked")

    return make_dimension("Security and supply-chain posture", score, evidence, notes)


def code_shape_dimension(ctx: RepoContext) -> DimensionResult:
    score = 55
    evidence: list[str] = []
    notes: list[str] = []
    files = product_code_files(ctx)

    if not files:
        evidence.append("no authored code files in scope")
        return make_dimension("Code shape and semantic surface", 65, evidence, notes)

    max_file = max_loc(files)
    small_files = sum(1 for file in files if file.line_count and file.line_count <= 300)
    if len(files) >= 5 and small_files / len(files) >= 0.7:
        score += 10
        evidence.append("most code files stay under 300 LOC")

    if has_prefix(ctx, "crates/domain") or has_prefix(ctx, "crates/application") or has_prefix(ctx, "crates/adapters"):
        score += 8
        evidence.append("layered crate layout present")

    if max_file > MEGAFILE_LOC:
        score -= 15
        evidence.append(f"code file exceeds {MEGAFILE_LOC} LOC")
    if max_file > VERY_LARGE_FILE_LOC:
        score -= 20
        evidence.append(f"code file exceeds {VERY_LARGE_FILE_LOC} LOC")

    huge_files = count_large_files(files, MEGAFILE_LOC)
    if huge_files > 1:
        score -= min(15, 5 * huge_files)
        notes.append(f"{huge_files} large code files in scope")

    large_functions = large_function_hits(ctx)
    if large_functions:
        worst = large_functions[0]
        score -= 12
        evidence.append(f"large function marker: {worst['path']}:{worst['line']}")

    duplicates = duplicate_blocks(ctx)
    if duplicates:
        score -= min(18, 6 + len(duplicates))
        evidence.append(f"duplicate code block marker: {duplicates[0]['path']}:{duplicates[0]['line']}")

    todos = todo_stub_hits(ctx)
    if todos:
        score -= min(20, 8 + len(todos))
        evidence.append(f"TODO/stub marker: {todos[0]['path']}:{todos[0]['line']}")

    fallbacks = fallback_soup_hits(ctx)
    if fallbacks:
        score -= min(18, 6 + len(fallbacks))
        evidence.append(f"fallback soup marker: {fallbacks[0]['path']}:{fallbacks[0]['line']}")

    future_hostile = future_hostile_dead_language_hits(ctx)
    if future_hostile:
        first = future_hostile[0]
        score -= min(24, 10 + len(future_hostile))
        evidence.append(
            f"future-hostile/dead-language marker: {first['path']}:{first['line']} `{first['matched_term']}`"
        )
        notes.append("product/runtime code contains future-hostile or dead-language terms")

    weak_names = weak_name_hits(ctx)
    if weak_names:
        score -= min(10, 3 + len(weak_names))
        evidence.append(f"weak name marker: {weak_names[0]['path']}")

    domain_io = [
        file.rel_path
        for file in files
        if any(part in {"core", "domain"} for part in file.rel_path.split("/"))
        and file.text
        and has_any(file.text, DOMAIN_IO_MARKERS)
    ]
    if domain_io:
        score -= 10
        evidence.append(f"IO markers found in domain/core files: {domain_io[0]}")

    junk_drawers = [
        file.rel_path
        for file in files
        if any(part in {"common", "helpers", "misc", "shared", "utils"} for part in file.rel_path.split("/"))
    ]
    if len(junk_drawers) > 5:
        score -= 8
        notes.append("junk-drawer directory names present")

    return make_dimension("Code shape and semantic surface", score, evidence, notes)


def data_truth_dimension(ctx: RepoContext) -> DimensionResult:
    score = 50
    evidence: list[str] = []
    notes: list[str] = []
    files = product_files(ctx)

    if has_prefix(ctx, "db") or any(file.rel_path.endswith(".sql") for file in files):
        score += 15
        evidence.append("database surface present")

    if has_prefix(ctx, "db/migrations") or has_prefix(ctx, "migrations"):
        score += 10
        evidence.append("migration directory present")

    if any(file.text and has_any(file.text, {"foreign key", "check constraint", "rls", "row level security"}) for file in files):
        score += 10
        evidence.append("constraint or RLS language found")

    if any(file.rel_path.startswith(("db/", "crates/adapters", "adapters", "infra", "data")) for file in files):
        score += 10
        evidence.append("data access appears compartmentalized")

    wrong_layer_db = [
        file.rel_path
        for file in files
        if file.text
        and (file.is_code or file.suffix == ".sql")
        and not file.rel_path.startswith(("db/", "migrations/", "adapters/", "infra/", "data/"))
        and file.rel_path not in {"tools/humanlint.py", "humanlint/cli.py"}
        and has_any(file.text, DB_MARKERS)
    ]
    if wrong_layer_db:
        score -= 15
        evidence.append(f"DB access found outside data layer: {wrong_layer_db[0]}")
        notes.append("direct DB access leaks out of the data boundary")

    strict_wrong_layer = wrong_layer_db_hits(ctx)
    if strict_wrong_layer:
        score -= 20
        evidence.append(f"strict DB boundary violation: {strict_wrong_layer[0]['path']}:{strict_wrong_layer[0]['line']}")

    if has_prefix(ctx, "db") and not has_prefix(ctx, "db/migrations") and has_any(lower_join(files), {"sqlx", "diesel", "prisma"}):
        score -= 10

    return make_dimension("Data truth and workflow safety", score, evidence, notes)


def observability_dimension(ctx: RepoContext) -> DimensionResult:
    score = 35
    evidence: list[str] = []
    notes: list[str] = []
    files = product_files(ctx)

    if any(file.text and has_any(file.text, OBSERVABILITY_MARKERS) for file in files):
        score += 15
        evidence.append("observability libraries or patterns found")

    if any(file.text and has_any(file.text, {"json diagnostics", "message-format=json", "stable error", "request id", "correlation id"}) for file in files):
        score += 10
        evidence.append("diagnostic shaping hints found")

    if has_prefix(ctx, "ops") or has_prefix(ctx, "observability"):
        score += 10
        evidence.append("ops/observability directory present")

    if any(file.text and has_any(file.text, {"receipt", "artifact", "raw log", "tee", "trace"}) for file in files):
        score += 8
        evidence.append("repair receipts or raw artifact language found")

    if has_agent_friendly_exceptions(ctx):
        score += 12
        evidence.append("agent-friendly exception pattern found")
    elif product_code_files(ctx, scope="all"):
        score -= 12
        notes.append("no agent-friendly exception pattern found")

    if any(file.text and has_any(file.text, {"println!", "console.log", "print("}) for file in files):
        score -= 8
        notes.append("free-form logging appears in scope")

    return make_dimension("Observability and repair evidence", score, evidence, notes)


def context_economy_dimension(ctx: RepoContext) -> DimensionResult:
    score = 20
    evidence: list[str] = []
    notes: list[str] = []

    root_agents = ctx.root_files.get("AGENTS.md")
    if root_agents:
        score += 25
        evidence.append("root `AGENTS.md` present")
        if root_agents.line_count <= 120 and root_agents.size <= 4096:
            score += 10
            evidence.append("root `AGENTS.md` stays short")
        else:
            score -= 10
            notes.append("root `AGENTS.md` is too long to route cleanly")
    else:
        notes.append("no root `AGENTS.md`")

    if any(file.rel_path in {"agent-map.json", "test-map.json", "proof-lanes.toml", "generated-zones.toml", "agent/owner-map.json"} for file in ctx.all_files):
        score += 10
        evidence.append("machine-readable routing artifacts present")

    local_agents = [file for file in ctx.all_files if file.name == "AGENTS.md" and file.rel_path != "AGENTS.md"]
    if local_agents:
        score += 10
        evidence.append("local instruction files present")

    readme = ctx.root_files.get("README.md")
    if root_readme_routes(readme):
        score += 10
        evidence.append("root README routes to the right docs")

    missing_docs = missing_core_docs(ctx)
    if missing_docs:
        score -= min(18, 6 * len(missing_docs))
        notes.append(f"missing agent-readable docs: {', '.join(missing_docs[:3])}")
    else:
        score += 8
        evidence.append("core agent-readable docs present")

    if not root_agents and not local_agents:
        score -= 10
        notes.append("no instruction files to route agents")

    return make_dimension("Context economy and agent instructions", score, evidence, notes)


def python_hygiene_dimension(ctx: RepoContext) -> DimensionResult:
    python_files = [file for file in ctx.scope_files if file.suffix == ".py"]
    non_optimal = non_optimal_language_hits(ctx)
    if not python_files:
        score = 100
        evidence = ["no Python files in scope"]
        notes: list[str] = []
        if non_optimal:
            score -= min(35, 10 + 3 * len(non_optimal))
            evidence.append(f"non-optimal product language marker: {non_optimal[0].rel_path}")
            notes.append("runtime code should converge to Rust, TypeScript, SQL, contracts, and bounded Python")
        return make_dimension(
            "Python containment and polyglot hygiene",
            score,
            evidence,
            notes,
        )

    score = 40
    evidence: list[str] = []
    notes: list[str] = []
    bad_paths = [file.rel_path for file in python_files if not is_allowed_python_path(file.rel_path)]
    if not bad_paths:
        score += 30
        evidence.append("Python stays inside allowed non-product roots")
    else:
        score -= 30
        evidence.append(f"Python appears outside the allowed roots: {bad_paths[0]}")
        notes.append("Python leaks into product surface")

    if any(file.rel_path.startswith("python/ai-service") for file in python_files):
        score += 10
        evidence.append("bounded AI/data service path present")

    ratio, python_lines, total_lines = python_product_ratio(ctx)
    if ratio > PYTHON_PRODUCT_RATIO_HARD:
        score -= 35
        evidence.append(f"Python is {ratio:.0%} of runtime product code")
        notes.append("too much Python for the selected optimal stack")
    elif ratio > PYTHON_PRODUCT_RATIO_SOFT:
        score -= 15
        evidence.append(f"Python is {ratio:.0%} of runtime product code")

    if non_optimal:
        score -= min(35, 10 + 3 * len(non_optimal))
        evidence.append(f"non-optimal product language marker: {non_optimal[0].rel_path}")
        notes.append("runtime code should converge to Rust, TypeScript, SQL, contracts, and bounded Python")

    db_python = [
        file.rel_path
        for file in python_files
        if bad_paths
        and file.text
        and has_any(file.text, {"psycopg", "sqlite3", "sqlalchemy", "postgres", "mysql", "sqlite"})
    ]
    if db_python and not any(path.startswith("python/ai-service") for path in db_python):
        score -= 20
        evidence.append(f"Python directly touches DB truth outside AI service: {db_python[0]}")

    return make_dimension("Python containment and polyglot hygiene", score, evidence, notes)


def build_speed_dimension(ctx: RepoContext) -> DimensionResult:
    score = 20
    evidence: list[str] = []
    notes: list[str] = []
    surface_text, _ = concat_command_surface(ctx)
    if has_any(surface_text, {"cargo check", "cargo nextest", "cargo build --timings", "sccache", "rust-cache", "bacon", "vitest", "pytest", "go test", "dotnet test", "pnpm", "bun test", "turbo", "nx"}):
        score += 20
        evidence.append("build acceleration markers found")

    if has_any(surface_text, {"cargo check", "nextest", "vitest", "pytest", "go test", "dotnet test"}):
        score += 10
        evidence.append("targeted test/build commands found")

    if any(file.name in {"Cargo.lock", "pnpm-lock.yaml", "package-lock.json", "yarn.lock", "uv.lock", "poetry.lock"} for file in ctx.all_files):
        score += 10
        evidence.append("locked dependency graph present")

    if any(file.text and has_any(file.text, {"actions/cache", "rust-cache", "sccache", "cache"}) for file in ctx.all_files if file.rel_path.startswith(".github/workflows")):
        score += 10
        evidence.append("CI cache hint found")

    if not has_one_command_setup(ctx):
        score -= 10
        notes.append("missing one-command setup/validation")

    if not has_fast_lane(ctx):
        score -= 10
        notes.append("missing deterministic fast lane")

    return make_dimension("Build speed signals", score, evidence, notes)


def scan_repo(ctx: RepoContext) -> tuple[list[DimensionResult], list[Finding], list[dict], list[str], int, int]:
    dimensions = [
        ownership_dimension(ctx),
        contract_dimension(ctx),
        proof_lanes_dimension(ctx),
        security_dimension(ctx),
        code_shape_dimension(ctx),
        data_truth_dimension(ctx),
        observability_dimension(ctx),
        context_economy_dimension(ctx),
        python_hygiene_dimension(ctx),
        build_speed_dimension(ctx),
    ]

    raw_score = int(round(sum(d.weighted_points for d in dimensions)))
    caps_applied: list[str] = []
    cap_limit = 100

    root_agents = ctx.root_files.get("AGENTS.md")
    if not root_agents and not any(file.rel_path.endswith("/AGENTS.md") for file in ctx.all_files):
        caps_applied.append("no-root-agent-instructions")
        cap_limit = min(cap_limit, 75)

    if not has_one_command_setup(ctx):
        caps_applied.append("no-one-command-setup-or-validation")
        cap_limit = min(cap_limit, 70)

    if not has_fast_lane(ctx):
        caps_applied.append("no-deterministic-fast-lane")
        cap_limit = min(cap_limit, 65)

    if is_high_risk_repo(ctx) and not has_security_lane(ctx):
        caps_applied.append("no-security-lane-on-high-risk-repo")
        cap_limit = min(cap_limit, 60)

    contract_surface = has_contract_surface(ctx) or has_polyglot_boundary(ctx)
    if contract_surface and not (has_generated_contracts(ctx) or has_api_drift_checks(ctx)):
        caps_applied.append("generated-contracts-or-public-api-drift-untested")
        cap_limit = min(cap_limit, 80)

    bad_python_paths = [
        file.rel_path
        for file in ctx.all_files
        if file.suffix == ".py" and file.is_generated is False and not is_allowed_python_path(file.rel_path)
    ]
    if bad_python_paths:
        caps_applied.append("python-direct-product-truth-or-db-ownership")
        cap_limit = min(cap_limit, 72)

    if is_high_risk_repo(ctx) and not has_secret_or_dependency_scans(ctx):
        caps_applied.append("no-secret-or-dependency-scanning-in-ci")
        cap_limit = min(cap_limit, 78)

    if is_high_risk_repo(ctx) and not has_humanlint_audit_ci_lane(ctx):
        caps_applied.append("no-humanlint-audit-lane-in-ci")
        cap_limit = min(cap_limit, 82)

    if non_optimal_language_hits(ctx):
        caps_applied.append("non-optimal-product-language-found")
        cap_limit = min(cap_limit, 74)

    ratio, _, _ = python_product_ratio(ctx)
    if ratio > PYTHON_PRODUCT_RATIO_SOFT:
        caps_applied.append("too-much-python-in-product-surface")
        cap_limit = min(cap_limit, 72)

    if todo_stub_hits(ctx):
        caps_applied.append("vibe-placeholders-in-product-code")
        cap_limit = min(cap_limit, 68)

    if fallback_soup_hits(ctx):
        caps_applied.append("fallback-soup-in-product-code")
        cap_limit = min(cap_limit, 70)

    if future_hostile_dead_language_hits(ctx):
        caps_applied.append("future-hostile-dead-language-in-product-code")
        cap_limit = min(cap_limit, 64)

    if duplicate_blocks(ctx):
        caps_applied.append("severe-duplication-in-product-code")
        cap_limit = min(cap_limit, 70)

    if generated_zone_issues(ctx):
        caps_applied.append("generated-zone-mutation-risk")
        cap_limit = min(cap_limit, 76)

    if wrong_layer_db_hits(ctx):
        caps_applied.append("direct-db-access-from-wrong-layer")
        cap_limit = min(cap_limit, 66)

    if has_web_surface(ctx) and not has_playwright_e2e(ctx):
        caps_applied.append("missing-web-e2e-lane")
        cap_limit = min(cap_limit, 82)

    ux_qa = ux_qa_status(ctx, has_web_surface(ctx))
    if ux_qa.web_surface and not ux_qa.has_rendered_ux_lane:
        caps_applied.append("missing-rendered-ux-qa-lane")
        cap_limit = min(cap_limit, 84)

    if has_rust_surface(ctx) and (not has_rust_property_tests(ctx) or not has_rust_integration_tests(ctx)):
        caps_applied.append("missing-rust-property-or-integration-tests")
        cap_limit = min(cap_limit, 82)

    if product_code_files(ctx, scope="all") and not has_agent_friendly_exceptions(ctx):
        caps_applied.append("no-agent-friendly-exception-pattern")
        cap_limit = min(cap_limit, 76)

    if missing_core_docs(ctx):
        caps_applied.append("missing-agent-readable-docs")
        cap_limit = min(cap_limit, 80)

    final_score = min(raw_score, cap_limit)
    findings = build_findings(ctx, dimensions, caps_applied)
    agent_fix_queue = build_agent_fix_queue(findings)
    return dimensions, findings, agent_fix_queue, caps_applied, raw_score, final_score


def build_findings(ctx: RepoContext, dimensions: list[DimensionResult], caps_applied: list[str]) -> list[Finding]:
    findings: list[Finding] = []
    dim_by_name = {d.name: d for d in dimensions}

    def add_finding(
        severity: str,
        category: str,
        path: str,
        problem: str,
        fix: str,
        evidence: list[str],
        line: int | None = None,
        matched_term: str | None = None,
        reason: str | None = None,
        rule_id: str | None = None,
    ) -> None:
        findings.append(
            Finding(
                severity=severity,
                category=category,
                path=path,
                problem=problem,
                agent_fix=fix,
                evidence=evidence,
                rule_id=rule_id,
                line=line,
                matched_term=matched_term,
                reason=reason,
            )
        )

    if "no-root-agent-instructions" in caps_applied:
        add_finding(
            "medium",
            "context",
            "AGENTS.md",
            "no root agent/developer instruction file routes contributors at the repository root",
            "add a concise root `AGENTS.md` and move deeper ownership rules into local docs",
            [
                "no root `AGENTS.md` detected",
                "no root routing map or ownership router found",
            ],
        )

    if "no-one-command-setup-or-validation" in caps_applied:
        add_finding(
            "high",
            "proof",
            ".",
            "no one-command setup or validation lane was detected",
            "add a canonical `setup`, `check`, `test`, or `verify` lane in one root command file",
            [
                "no root setup/check/test/verify target surfaced",
                "no canonical validation lane found in root automation files",
            ],
        )

    if "no-deterministic-fast-lane" in caps_applied:
        add_finding(
            "high",
            "proof",
            ".",
            "no deterministic fast lane was detected",
            "add a fast lane that runs the narrowest deterministic proof loop and keep it canonical",
            [
                "no fast lane markers found",
                "no targeted fast validation command surfaced",
            ],
            rule_id="HLT-004-UNMAPPED-PROOF",
        )

    if "no-security-lane-on-high-risk-repo" in caps_applied:
        add_finding(
            "high",
            "security",
            ".github/workflows",
            "high-risk repo has no explicit security lane",
            "add a dedicated security lane with secret scanning, dependency review, and workflow linting",
            [
                "no security lane markers found",
                "repo contains executable/code surface",
            ],
            rule_id="HLT-009-GENERATED-SECURITY",
        )

    if "generated-contracts-or-public-api-drift-untested" in caps_applied:
        add_finding(
            "high",
            "boundary",
            "contracts/",
            "generated contracts or public API drift are not being checked",
            "generate boundary clients and gate drift with public-API or semver checks",
            [
                "contract surface exists",
                "no generated contract evidence or API drift check found",
            ],
            rule_id="HLT-007-HANDWRITTEN-CONTRACT",
        )

    if "python-direct-product-truth-or-db-ownership" in caps_applied:
        bad = [
            file.rel_path
            for file in ctx.all_files
            if file.suffix == ".py" and not is_allowed_python_path(file.rel_path)
        ]
        add_finding(
            "high",
            "python",
            bad[0] if bad else "python/",
            "Python appears outside the bounded AI/data service or owns product truth",
            "move Python into `python/ai-service` or keep it tooling-only under `tools/`",
            [
                bad[0] if bad else "python path outside allowed roots",
                "Python should stay away from product truth and production DB ownership",
            ],
            rule_id="HLT-005-PYTHON-PRODUCT-TRUTH",
        )

    if "no-secret-or-dependency-scanning-in-ci" in caps_applied:
        add_finding(
            "high",
            "security",
            ".github/workflows",
            "no secret or dependency scanning was found in CI",
            "add secret scanning, dependency review, and SBOM or provenance checks to CI",
            [
                "no CI scan markers found",
                "no secret/dependency scanning surfaced in automation",
            ],
            rule_id="HLT-010-SECRET-SPRAWL",
        )

    if "no-humanlint-audit-lane-in-ci" in caps_applied:
        add_finding(
            "high",
            "audit",
            ".github/workflows",
            "CI does not run the humanlint audit lane",
            "add a CI job that runs `python3 tools/humanlint.py . --json repo-score.json --md repo-score.md` and uploads both artifacts",
            [
                "no humanlint audit marker found in workflow files",
                "audit output must stay JSON plus Markdown for agent repair routing",
            ],
        )

    non_optimal = non_optimal_language_hits(ctx)
    if non_optimal:
        first = non_optimal[0]
        add_finding(
            "high",
            "stack",
            first.rel_path,
            "runtime code uses a language outside the chosen optimal stack",
            "move product runtime behavior to Rust core, TypeScript web, SQL migrations, generated contracts, or bounded `python/ai-service` only",
            [
                f"{first.rel_path} uses `{first.suffix}`",
                TARGET_STACK,
            ],
        )

    ratio, python_lines, total_lines = python_product_ratio(ctx)
    if ratio > PYTHON_PRODUCT_RATIO_SOFT:
        add_finding(
            "high" if ratio > PYTHON_PRODUCT_RATIO_HARD else "medium",
            "python",
            "python/ai-service",
            "Python is too large a share of runtime product code for this standard",
            "keep Python bounded to model/data work and move durable product truth, authz, workflows, and core behavior into Rust",
            [
                f"Python runtime LOC: {python_lines}",
                f"Total runtime LOC: {total_lines}",
                f"Python share: {ratio:.0%}",
            ],
        )

    todos = todo_stub_hits(ctx)
    if todos:
        first = todos[0]
        add_finding(
            "high",
            "vibe",
            first["path"],
            "product code contains TODO/stub/unimplemented/unreachable placeholder markers",
            "replace placeholders with implemented behavior, typed unsupported-state errors, or a tracked exception record with docs",
            [
                f"{first['path']}:{first['line']} {first['text']}",
                "placeholders are hard failures in agent-native product code",
            ],
            rule_id="HLT-001-DEAD-MARKER",
        )

    fallbacks = fallback_soup_hits(ctx)
    if fallbacks:
        first = fallbacks[0]
        add_finding(
            "high",
            "vibe",
            first["path"],
            "fallback soup detected in product code",
            "collapse fallback chains into explicit typed states with bounded retry policy, telemetry, and documented repair guidance",
            [
                f"{first['path']}:{first['line']} {first['text']}",
                f"{len(fallbacks)} fallback marker(s) found in scoped runtime code",
            ],
            rule_id="HLT-001-DEAD-MARKER",
        )

    future_hostile = future_hostile_dead_language_hits(ctx)
    if future_hostile:
        for hit in future_hostile:
            add_finding(
                "high",
                "vibe",
                hit["path"],
                f"future-hostile/dead-language term `{hit['matched_term']}` appears in product/runtime code",
                hit["agent_fix"],
                [
                    f"{hit['path']}:{hit['line']} matched `{hit['matched_term']}`",
                    hit["text"],
                ],
                line=hit["line"],
                matched_term=hit["matched_term"],
                reason=hit["reason"],
                rule_id="HLT-001-DEAD-MARKER",
            )

    duplicates = duplicate_blocks(ctx)
    if duplicates:
        first = duplicates[0]
        add_finding(
            "high",
            "vibe",
            first["path"],
            "duplicated product code block detected",
            "extract the duplicated behavior behind one named boundary and add focused tests before changing behavior",
            [
                f"{first['path']}:{first['line']} {first['text']}",
                f"{len(duplicates)} duplicate block marker(s) found",
            ],
        )

    generated_issues = generated_zone_issues(ctx)
    if generated_issues:
        first = generated_issues[0]
        add_finding(
            "high",
            "generated",
            first["path"],
            "generated zone is not protected strongly enough against hand edits",
            "add `agent/generated-zones.toml`, require generated/do-not-edit markers, and route repairs to the source contract",
            [
                f"{first['path']}:{first['line']} {first['text']}",
                "generated files must be repaired from source contracts, not edited by hand",
            ],
            rule_id="HLT-002-GENERATED-MUTATION",
        )

    strict_db = wrong_layer_db_hits(ctx)
    if strict_db and not any(item.category == "data" for item in findings):
        first = strict_db[0]
        add_finding(
            "high",
            "data",
            first["path"],
            "direct database access appears in a wrong layer",
            "move SQL and DB clients to `crates/adapters` or `db/migrations`; expose typed application/domain APIs upward",
            [
                f"{first['path']}:{first['line']} {first['text']}",
                "UI, API handlers, domain, and application layers must not own raw DB access",
            ],
            rule_id="HLT-006-DIRECT-DB-WRONG-LAYER",
        )

    if "missing-web-e2e-lane" in caps_applied:
        add_finding(
            "high",
            "test",
            "apps/web",
            "web surface lacks a Playwright/Cypress e2e lane",
            "add Playwright e2e tests for critical user flows and wire them into the fast or CI proof map",
            [
                "web surface detected",
                "no Playwright/Cypress/e2e marker found",
            ],
            rule_id="HLT-013-RENDERED-UX-GAP",
        )

    if "missing-rendered-ux-qa-lane" in caps_applied:
        ux_qa = ux_qa_status(ctx, True)
        add_finding(
            "high",
            "ux-qa",
            "apps/web",
            "web surface lacks layered rendered UX QA evidence",
            "add Storybook state coverage, Playwright screenshots, visual review or `@humanlint/ux-qa`, accessibility scans, CLS checks, generated mocks, and design tokens",
            ux_qa.missing_categories or ["rendered UX QA lane missing"],
            rule_id="HLT-013-RENDERED-UX-GAP",
        )

    if "missing-rust-property-or-integration-tests" in caps_applied:
        missing = []
        if not has_rust_property_tests(ctx):
            missing.append("property tests")
        if not has_rust_integration_tests(ctx):
            missing.append("integration tests")
        add_finding(
            "high",
            "test",
            "crates/",
            "Rust surface lacks required property and/or integration tests",
            "add `proptest` or equivalent invariant tests plus `tests/` integration coverage routed through `cargo nextest` or `cargo test`",
            [
                "Rust surface detected",
                f"missing: {', '.join(missing)}",
            ],
            rule_id="HLT-008-FALSE-GREEN-RISK",
        )

    if "no-agent-friendly-exception-pattern" in caps_applied:
        add_finding(
            "high",
            "exceptions",
            "crates/domain",
            "no agent-friendly exception/error pattern was detected",
            "define typed errors with stable code, purpose, reason, common fixes, and documentation URL; mirror the shape in TypeScript boundaries",
            [
                "no error type with purpose/reason/common fixes/docs URL markers found",
                "agents need structured repair hints instead of opaque failures",
            ],
            rule_id="HLT-017-OPAQUE-OBSERVABILITY",
        )

    if "missing-agent-readable-docs" in caps_applied:
        missing = missing_core_docs(ctx)
        add_finding(
            "medium",
            "docs",
            "docs/",
            "agent-readable documentation is incomplete",
            "add concise docs for architecture, boundaries, tests, generated zones, and audit rules; route them from root `AGENTS.md`",
            missing[:5] or ["missing core docs"],
        )

    ownership = dim_by_name["Ownership and navigation surface"]
    if ownership.score < 55 and not any(item.category == "context" for item in findings):
        add_finding(
            "medium",
            "context",
            ".",
            "navigation surface is thin for agent work",
            "add local routing docs and machine-readable owner/test maps where the repo needs them",
            ownership.evidence[:2] or ["ownership/navigation signals are weak"],
        )

    code_shape = dim_by_name["Code shape and semantic surface"]
    if code_shape.notes and any("large code files" in note for note in code_shape.notes):
        biggest = max_loc(ctx.scope_authored_code_files)
        add_finding(
            "medium" if biggest <= VERY_LARGE_FILE_LOC else "high",
            "shape",
            next((file.rel_path for file in ctx.scope_authored_code_files if file.line_count == biggest), "."),
            f"largest code file is {biggest} LOC",
            "split the file along ownership or semantic boundaries before agents have to patch it again",
            [f"largest authored code file: {biggest} LOC"],
        )

    large_functions = large_function_hits(ctx)
    if large_functions:
        first = large_functions[0]
        add_finding(
            "medium",
            "shape",
            first["path"],
            "large function exceeds the agent-native review budget",
            "split the function into named pure decisions, boundary adapters, and testable helpers; keep functions under 80 LOC by default",
            [f"{first['path']}:{first['line']} {first['text']}"],
        )

    weak_names = weak_name_hits(ctx)
    if weak_names:
        first = weak_names[0]
        add_finding(
            "medium",
            "naming",
            first["path"],
            "weak generic names make ownership and intent ambiguous",
            "rename generic helpers/utils/common/manager surfaces around the exact domain decision or adapter they own",
            [f"{first['path']}: {first['text']}"],
        )

    data = dim_by_name["Data truth and workflow safety"]
    if any("DB access" in item for item in data.evidence) and not any(item.category == "data" for item in findings):
        add_finding(
            "high",
            "data",
            data.evidence[-1].split(": ", 1)[-1] if data.evidence else ".",
            "database access appears outside the dedicated data boundary",
            "move writes and SQL into adapters or the db layer and keep domain/UI code out of it",
            data.evidence[-2:] or ["DB access boundary is blurred"],
        )

    return findings


def build_agent_fix_queue(findings: list[Finding]) -> list[dict]:
    queue: list[dict] = []
    seen: set[tuple[str, str]] = set()
    for finding in findings:
        key = (finding.path, finding.agent_fix)
        if key in seen:
            continue
        seen.add(key)
        queue.append(
            {
                "path": finding.path,
                "priority": finding.severity,
                "rule_id": finding.rule_id,
                "task": finding.agent_fix,
                "why": finding.problem,
            }
        )
    return queue


def report_to_dict(
    ctx: RepoContext,
    dimensions: list[DimensionResult],
    findings: list[Finding],
    agent_fix_queue: list[dict],
    caps_applied: list[str],
    raw_score: int,
    final_score: int,
) -> dict:
    return {
        "standard": "humanlint",
        "standard_version": STANDARD_VERSION,
        "auditor_version": AUDITOR_VERSION,
        "schema_version": SCHEMA_VERSION,
        "paper_edition": PAPER_EDITION,
        "target_stack_id": TARGET_STACK_ID,
        "target_stack": TARGET_STACK,
        "repo": str(ctx.root),
        "scope": {
            "mode": ctx.scope_mode,
            "paths": ctx.scope_paths,
        },
        "score": final_score,
        "raw_score": raw_score,
        "caps_applied": caps_applied,
        "hard_rules": [
            {
                "id": rule_id,
                "max_score": max_score,
            }
            for rule_id, max_score in CAPS
        ],
        "dimensions": [
            {
                "name": dim.name,
                "weight": dim.weight,
                "score": dim.score,
                "weighted_points": dim.weighted_points,
                "evidence": dim.evidence,
                "notes": dim.notes,
            }
            for dim in dimensions
        ],
        "ux_qa": ux_qa_status(ctx, has_web_surface(ctx)).as_dict(),
        "findings": [finding.as_dict() for finding in findings],
        "agent_fix_queue": agent_fix_queue,
    }


def render_markdown(report: dict) -> str:
    lines: list[str] = []
    lines.append("# humanlint Repo Score")
    lines.append("")
    lines.append(f"- Standard: `{report.get('standard', 'humanlint')} {report.get('standard_version', '')}`")
    lines.append(f"- Auditor: `{report.get('auditor_version', '')}`")
    lines.append(f"- Schema: `{report.get('schema_version', '')}`")
    lines.append(f"- Paper edition: `{report.get('paper_edition', '')}`")
    lines.append(f"- Target stack ID: `{report.get('target_stack_id', '')}`")
    lines.append(f"- Target stack: `{report.get('target_stack', TARGET_STACK)}`")
    lines.append(f"- Repo: `{report['repo']}`")
    scope = report.get("scope", {})
    lines.append(f"- Scope: `{scope.get('mode', 'full')}`")
    if scope.get("paths"):
        lines.append(f"- Changed: `{', '.join(scope['paths'])}`")
    lines.append(f"- Raw score: `{report['raw_score']}`")
    lines.append(f"- Final score: `{report['score']}`")
    caps = report.get("caps_applied", [])
    lines.append(f"- Caps applied: `{', '.join(caps) if caps else 'none'}`")
    lines.append("")
    lines.append("## Hard Rule Caps")
    lines.append("")
    lines.append("| Rule | Max Score | Applied |")
    lines.append("| --- | ---: | --- |")
    applied = set(caps)
    for rule in report.get("hard_rules", []):
        mark = "yes" if rule["id"] in applied else "no"
        lines.append(f"| `{rule['id']}` | {rule['max_score']} | {mark} |")
    lines.append("")
    lines.append("## Dimensions")
    lines.append("")
    lines.append("| Dimension | Weight | Score | Weighted | Evidence |")
    lines.append("| --- | ---: | ---: | ---: | --- |")
    for dim in report["dimensions"]:
        evidence = "; ".join(dim["evidence"][:2]) if dim["evidence"] else ""
        lines.append(
            f"| {dim['name']} | {dim['weight']} | {dim['score']} | {dim['weighted_points']:.2f} | {evidence} |"
        )
    lines.append("")
    ux_qa = report.get("ux_qa", {})
    if ux_qa:
        lines.append("## Rendered UX QA")
        lines.append("")
        lines.append(f"- Web surface: `{ux_qa.get('web_surface', False)}`")
        lines.append(f"- Layered UX lane: `{ux_qa.get('has_rendered_ux_lane', False)}`")
        missing = ux_qa.get("missing_categories") or []
        lines.append(f"- Missing: `{', '.join(missing) if missing else 'none'}`")
        lines.append("")
    lines.append("## Findings")
    lines.append("")
    if report["findings"]:
        for idx, finding in enumerate(report["findings"], start=1):
            location = finding["path"]
            if finding.get("line"):
                location = f"{location}:{finding['line']}"
            lines.append(f"{idx}. `{finding['severity']}` `{finding['category']}` `{location}`")
            if finding.get("rule_id"):
                lines.append(f"   Rule: `{finding['rule_id']}`")
            if finding.get("matched_term"):
                lines.append(f"   Matched term: `{finding['matched_term']}`")
            lines.append(f"   Reason: {finding.get('reason') or finding['problem']}")
            lines.append(f"   Fix: {finding['agent_fix']}")
            if finding["evidence"]:
                lines.append(f"   Evidence: {', '.join(finding['evidence'])}")
    else:
        lines.append("No findings.")
    lines.append("")
    lines.append("## Agent Fix Queue")
    lines.append("")
    if report["agent_fix_queue"]:
        for idx, item in enumerate(report["agent_fix_queue"], start=1):
            rule = f" `{item['rule_id']}`" if item.get("rule_id") else ""
            lines.append(f"{idx}. `{item['priority']}`{rule} `{item['path']}` - {item['task']}")
    else:
        lines.append("No queued fixes.")
    return "\n".join(lines) + "\n"


def write_output(path: str | None, content: str) -> None:
    if path is None:
        return
    if path == "-":
        sys.stdout.write(content)
        return
    Path(path).write_text(content, encoding="utf-8")


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Score a repo with the humanlint rubric.")
    parser.add_argument("repo", nargs="?", default=".", help="repo root to score")
    parser.add_argument(
        "--changed",
        nargs="*",
        default=[],
        help="optional changed file or directory paths to narrow file-scoped inspection",
    )
    parser.add_argument("--json", default="repo-score.json", help="JSON output path, or - for stdout")
    parser.add_argument("--md", default="repo-score.md", help="Markdown output path, or - for stdout")
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    root = Path(args.repo).expanduser().resolve()
    if not root.exists() or not root.is_dir():
        print(f"repo root not found: {root}", file=sys.stderr)
        return 2

    all_files = inventory_repo(root)
    changed = []
    for item in args.changed:
        rel = norm_relpath(root, item)
        if rel:
            changed.append(rel)

    ctx = build_context(root, all_files, changed)
    dimensions, findings, agent_fix_queue, caps_applied, raw_score, final_score = scan_repo(ctx)
    report = report_to_dict(ctx, dimensions, findings, agent_fix_queue, caps_applied, raw_score, final_score)

    json_text = json.dumps(report, indent=2, ensure_ascii=True)
    md_text = render_markdown(report)

    write_output(args.json, json_text + "\n")
    write_output(args.md, md_text)

    if args.json is None and args.md is None:
        print(md_text, end="")
    else:
        print(
            f"score={final_score} raw={raw_score} caps={len(caps_applied)} findings={len(findings)}",
            file=sys.stderr,
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
