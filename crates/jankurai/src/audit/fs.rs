use crate::model::FileInfo;
use anyhow::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

const EXCLUDED_DIRS: &[&str] = &[
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
];

const EXCLUDED_AGENT_STATE_DIRS: &[&str] = &[".antigravity", "antigravity"];
const CURSOR_ALLOWED_PREFIXES: &[&str] = &[".cursor/rules/"];

const TEXT_BASENAMES: &[&str] = &[
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
];

const TEXT_EXTS: &[&str] = &[
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
];

const CODE_EXTS: &[&str] = &[
    ".c", ".cc", ".cpp", ".cs", ".dart", ".go", ".h", ".hh", ".hpp", ".java", ".js", ".jsx", ".kt",
    ".kts", ".ex", ".exs", ".lua", ".m", ".mm", ".py", ".php", ".rb", ".rs", ".sh", ".swift",
    ".scala", ".ts", ".tsx",
];

const MAX_CAPTURE_CHARS: usize = 120_000;

pub fn inventory_repo(root: &Path) -> Result<Vec<FileInfo>> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_exclude(true)
        .git_global(true)
        .max_depth(None)
        .build()
    {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let rel = match path.strip_prefix(root) {
            Ok(rel) => rel,
            Err(_) => continue,
        };
        if should_skip(rel) {
            continue;
        }
        paths.push(rel.to_path_buf());
    }
    paths.sort();

    let mut files = Vec::with_capacity(paths.len());
    for rel in paths {
        let abs = root.join(&rel);
        let meta = match abs.metadata() {
            Ok(meta) => meta,
            Err(_) => continue,
        };
        let rel_path = rel.to_string_lossy().replace('\\', "/");
        let name = abs
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let suffix = suffix_of(&rel_path);
        let is_code = is_code_file(&name, &suffix);
        let (text, line_count) = if is_text_candidate(&name, &suffix, &rel_path) {
            read_text_sample(&abs)?
        } else {
            (String::new(), 0)
        };
        let is_generated = rel_path.split('/').any(|part| {
            part == "generated"
                || part.starts_with("generated")
                || part == "gen"
                || part == "artifacts"
        });
        files.push(FileInfo {
            rel_path,
            name,
            suffix,
            size: meta.len(),
            line_count,
            text,
            is_generated,
            is_code,
        });
    }
    Ok(files)
}

fn should_skip(path: &Path) -> bool {
    let rel = path.to_string_lossy().replace('\\', "/");
    if rel.starts_with(".cursor/") && !CURSOR_ALLOWED_PREFIXES.iter().any(|p| rel.starts_with(p)) {
        return true;
    }
    if EXCLUDED_AGENT_STATE_DIRS
        .iter()
        .any(|dir| rel == *dir || rel.starts_with(&format!("{dir}/")))
    {
        return true;
    }
    path.components().any(|c| {
        let s = c.as_os_str().to_string_lossy();
        EXCLUDED_DIRS.contains(&s.as_ref())
    })
}

fn suffix_of(rel_path: &str) -> String {
    let lower = rel_path.to_ascii_lowercase();
    if lower.ends_with(".d.ts") {
        ".d.ts".to_string()
    } else {
        Path::new(rel_path)
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| format!(".{}", s.to_ascii_lowercase()))
            .unwrap_or_default()
    }
}

fn is_text_candidate(name: &str, suffix: &str, rel_path: &str) -> bool {
    let lower = rel_path.to_ascii_lowercase();
    TEXT_BASENAMES
        .iter()
        .any(|item| item.eq_ignore_ascii_case(name))
        || TEXT_EXTS.iter().any(|ext| lower.ends_with(ext))
        || matches!(suffix, ".dockerfile")
        || matches!(
            name.to_ascii_lowercase().as_str(),
            "dockerfile" | "makefile" | "justfile"
        )
}

fn is_code_file(name: &str, suffix: &str) -> bool {
    matches!(name, "Makefile" | "makefile" | "Justfile" | "justfile") || CODE_EXTS.contains(&suffix)
}

fn read_text_sample(path: &Path) -> Result<(String, usize)> {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let mut line_count = 0usize;
    let mut captured = String::new();
    for line in text.lines() {
        line_count += 1;
        if captured.len() < MAX_CAPTURE_CHARS {
            let remaining = MAX_CAPTURE_CHARS - captured.len();
            let mut piece = line.as_bytes();
            if piece.len() > remaining {
                piece = &piece[..remaining];
            }
            captured.push_str(std::str::from_utf8(piece).unwrap_or_default());
            captured.push('\n');
        }
    }
    Ok((captured, line_count))
}
