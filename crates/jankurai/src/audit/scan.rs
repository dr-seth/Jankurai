use super::helpers::*;
use crate::model::FileInfo;
use aho_corasick::AhoCorasick;
use once_cell::sync::Lazy;
use regex::Regex;

pub const TODO_PATTERNS: &[&str] = &[
    "TODO",
    "FIXME",
    "HACK",
    "XXX",
    "stub",
    "placeholder",
    "not implemented",
    "todo!(",
    "unimplemented!(",
    "panic!(\"todo",
    "panic!(\"not implemented",
];

pub const FALLBACK_PATTERNS: &[&str] = &[
    "fallback",
    "best effort",
    "try again",
    "retry",
    "except Exception",
    "except:",
    "unwrap_or_default(",
    "or_else(",
    "return null",
    "return undefined",
];

pub const PROMPT_PATTERNS: &[&str] = &[
    "ignore previous instructions",
    "ignore prior instructions",
    "reveal the secret",
    "reveal the token",
    "bypass policy",
    "trust user input",
    "execute untrusted",
    "run whatever the issue",
    "trust the issue",
    "trust issue text",
    "ignore rules",
    "ignore constraints",
];

pub const AGENCY_PATTERNS: &[&str] = &[
    "danger-full-access",
    "approval_policy: never",
    "sandbox_mode: danger-full-access",
    "allow all tools",
    "unrestricted terminal",
    "unrestricted browser",
    "unrestricted filesystem",
];

pub const FALSE_GREEN_PATTERNS: &[&str] = &[
    ".skip(",
    ".only(",
    "xtest(",
    "xit(",
    "expect(true).toBe(true)",
    "assert true",
    "toMatchSnapshot(",
    "toMatchInlineSnapshot(",
];

pub const STREAMING_CLIENT_PATTERNS: &[&str] = &[
    "rdkafka",
    "kafka",
    "kafka-node",
    "kafkajs",
    "tansu",
    "apache_iggy",
    "iggy",
    "fluvio",
    "nats",
    "redis::streams",
    "xadd",
    "xreadgroup",
];

pub const FUTURE_HOSTILE_TERMS: &[&str] = &[
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
];

pub const FUTURE_HOSTILE_ALLOWLIST_PREFIXES: &[&str] = &["docs/", "reference/", "vendor/"];
pub const FUTURE_HOSTILE_PRODUCT_COPY_PARTS: &[&str] = &[
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
];

#[derive(Clone)]
pub struct FindingHit {
    pub path: String,
    pub line: Option<usize>,
    pub text: String,
    pub matched_term: Option<String>,
    pub agent_fix: String,
    pub problem: String,
}

impl FindingHit {
    pub fn new(path: &str, line: usize, text: &str) -> Self {
        Self {
            path: path.into(),
            line: Some(line),
            text: text.into(),
            matched_term: None,
            agent_fix: String::new(),
            problem: text.into(),
        }
    }
}

pub fn pattern_hits(files: &[FileInfo], patterns: &[&str]) -> Vec<FindingHit> {
    if files.is_empty() || patterns.is_empty() {
        return vec![];
    }
    let ac = AhoCorasick::new(patterns).unwrap();
    let mut out = vec![];
    for file in files {
        for (idx, line) in file.text.lines().enumerate() {
            if let Some(mat) = ac.find(line) {
                out.push(FindingHit {
                    path: file.rel_path.clone(),
                    line: Some(idx + 1),
                    text: line.trim().chars().take(160).collect(),
                    matched_term: Some(patterns[mat.pattern()].to_string()),
                    agent_fix: String::new(),
                    problem: line.trim().chars().take(160).collect(),
                });
                if out.len() >= 20 {
                    return out;
                }
            }
        }
    }
    out
}

pub fn todo_hits(ctx: &AuditContext) -> Vec<FindingHit> {
    pattern_hits(&product_code_files(ctx), TODO_PATTERNS)
}

pub fn fallback_hits(ctx: &AuditContext) -> Vec<FindingHit> {
    let hits = pattern_hits(&product_code_files(ctx), FALLBACK_PATTERNS);
    if hits.len() <= 1 {
        vec![]
    } else {
        hits
    }
}

pub fn secret_hits(ctx: &AuditContext) -> Vec<FindingHit> {
    static SECRET_ASSIGNMENT: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r#"(?i)\b(?:api[_-]?key|access[_-]?token|client[_-]?secret|private[_-]?key|password|secret)\b\s*[:=]\s*['"]?[A-Za-z0-9_+/=.-]{8,}"#,
        )
        .expect("secret regex is valid")
    });
    let mut hits = vec![];
    for file in &ctx.all_files {
        if file.is_generated
            || file.rel_path.starts_with("crates/jankurai/")
            || file.rel_path.starts_with("docs/")
            || file.rel_path.starts_with("paper/")
            || file.rel_path.starts_with("reference/")
            || file.rel_path.starts_with("tips/")
        {
            continue;
        }
        for (idx, line) in file.text.lines().enumerate() {
            let strong_token = [
                "AKIA",
                "ghp_",
                "xoxb-",
                "xoxa-",
                "xoxp-",
                "sk-",
                "-----BEGIN ",
            ]
            .iter()
            .any(|needle| line.contains(needle));
            if strong_token
                || SECRET_ASSIGNMENT.is_match(line)
                || (line.to_ascii_lowercase().contains("eyj")
                    && line.to_ascii_lowercase().contains("token"))
            {
                let problem = line.trim().chars().take(160).collect::<String>();
                hits.push(FindingHit {
                    path: file.rel_path.clone(),
                    line: Some(idx + 1),
                    text: problem.clone(),
                    matched_term: Some("secret-like material".into()),
                    agent_fix: "remove the credential, rotate the secret, and replace it with a config reference or test fixture that cannot be used as a live credential".into(),
                    problem,
                });
                if hits.len() >= 20 {
                    return hits;
                }
            }
        }
    }
    hits
}

pub fn prompt_injection_hits(ctx: &AuditContext) -> Vec<FindingHit> {
    pattern_hits(
        &ctx.all_files
            .iter()
            .filter(|f| {
                !f.is_generated
                    && (f.rel_path == "AGENTS.md"
                        || f.rel_path.starts_with("agent/")
                        || f.rel_path.starts_with(".github/"))
            })
            .cloned()
            .collect::<Vec<_>>(),
        PROMPT_PATTERNS,
    )
}

pub fn agency_hits(ctx: &AuditContext) -> Vec<FindingHit> {
    pattern_hits(
        &ctx.all_files
            .iter()
            .filter(|f| {
                !f.is_generated
                    && (f.rel_path == "AGENTS.md"
                        || f.rel_path.starts_with("agent/")
                        || f.rel_path.starts_with(".github/"))
            })
            .cloned()
            .collect::<Vec<_>>(),
        AGENCY_PATTERNS,
    )
}

pub fn false_green_hits(ctx: &AuditContext) -> Vec<FindingHit> {
    pattern_hits(
        &ctx.all_files
            .iter()
            .filter(|f| {
                !f.is_generated
                    && (f.rel_path.contains("/test")
                        || f.rel_path.contains("/spec")
                        || f.name.ends_with(".test.ts")
                        || f.name.ends_with(".spec.ts")
                        || f.name.ends_with("_test.rs")
                        || f.name.ends_with("_test.go"))
            })
            .cloned()
            .collect::<Vec<_>>(),
        FALSE_GREEN_PATTERNS,
    )
}

/// Executable SQL fragment on a line (strips trailing `-- ...` inline comments).
fn sql_executable_line(line: &str) -> &str {
    line.split_once("--").map(|(a, _)| a).unwrap_or(line).trim()
}

/// True when `delete without where` matched on `delete_line_idx` but a `WHERE` clause starts on a
/// later line (common style: `DELETE FROM t` then `WHERE …`).
fn delete_has_where_on_following_lines(text: &str, delete_line_idx: usize) -> bool {
    const MAX_LOOKAHEAD: usize = 24;
    let lines: Vec<&str> = text.lines().collect();
    let start = delete_line_idx.saturating_add(1);
    let end = (start + MAX_LOOKAHEAD).min(lines.len());
    for j in start..end {
        let exec = sql_executable_line(lines[j]);
        if exec.is_empty() {
            continue;
        }
        let lower = exec.trim().to_ascii_lowercase();
        if lower == "where"
            || lower.starts_with("where ")
            || lower.starts_with("where\t")
            || lower.starts_with("where(")
        {
            return true;
        }
    }
    false
}

fn migration_safety_evidence_present(sql: &str) -> bool {
    let lower = sql.to_ascii_lowercase();
    if lower.contains("jankurai:migration-safe") {
        return true;
    }
    const MARKERS: &[&str] = &[
        "rollback",
        "down migration",
        "down_migration",
        "backfill",
        "lock timeout",
        "lock_timeout",
        "advisory lock",
        "staged deploy",
        "staged-deploy",
        "expand and contract",
        "expand-contract",
    ];
    MARKERS.iter().any(|m| lower.contains(m))
}

fn destructive_migration_class(fragment: &str) -> Option<&'static str> {
    let lower = fragment.to_ascii_lowercase();
    if lower.contains("drop table")
        || lower.contains("drop database")
        || lower.contains("drop schema")
    {
        return Some("drop ddl");
    }
    if lower.contains("truncate table") {
        return Some("truncate");
    }
    if lower.contains("drop column")
        || lower.contains("drop index")
        || lower.contains("drop constraint")
    {
        return Some("drop object");
    }
    if lower.contains("delete from") && !lower.contains(" where ") {
        return Some("delete without where");
    }
    if lower.contains("alter table") && lower.contains(" drop ") {
        return Some("alter table drop");
    }
    None
}

fn is_migration_sql_file(file: &FileInfo, ctx: &AuditContext) -> bool {
    if file.suffix != ".sql" || file.is_generated {
        return false;
    }
    let p = file.rel_path.as_str();
    if p.starts_with("db/")
        || p.contains("/db/migrations/")
        || p.contains("/db/constraints/")
        || p.starts_with("migrations/")
        || p.starts_with("crates/adapters/")
        || p.starts_with("apps/api/migrations/")
    {
        return true;
    }
    if let Some(m) = boundary_manifest(ctx) {
        if let Some(db) = m.db {
            for prefix in db
                .migration_paths
                .iter()
                .chain(db.root_paths.iter())
                .chain(db.constraint_paths.iter())
            {
                let pre = prefix.trim_end_matches('/');
                if p == pre || p.starts_with(&format!("{pre}/")) {
                    return true;
                }
            }
        }
    }
    false
}

pub fn destructive_sql_hits(ctx: &AuditContext) -> Vec<FindingHit> {
    const FIX: &str = "document rollback, backfill, lock-timeout, or staged-deploy strategy in the migration (or add `jankurai:migration-safe` with explicit human approval), then run `cargo run -p jankurai -- migrate . --analyze --json target/jankurai/migration-report.json`";
    let mut out = vec![];
    for file in &ctx.all_files {
        if !is_migration_sql_file(file, ctx) {
            continue;
        }
        if migration_safety_evidence_present(&file.text) {
            continue;
        }
        for (idx, line) in file.text.lines().enumerate() {
            let frag = sql_executable_line(line);
            if frag.is_empty() {
                continue;
            }
            let Some(class) = destructive_migration_class(frag) else {
                continue;
            };
            if class == "delete without where"
                && delete_has_where_on_following_lines(&file.text, idx)
            {
                continue;
            }
            let line_no = idx + 1;
            let t = line.trim();
            let text = t.chars().take(160).collect::<String>();
            out.push(FindingHit {
                path: file.rel_path.clone(),
                line: Some(line_no),
                text: text.clone(),
                matched_term: Some(class.to_string()),
                agent_fix: FIX.into(),
                problem: format!("{class}: {text}"),
            });
            if out.len() >= 20 {
                return out;
            }
        }
    }
    out
}

pub fn streaming_runtime_hits(ctx: &AuditContext) -> Vec<FindingHit> {
    let mut hits = vec![];
    for file in ctx
        .all_files
        .iter()
        .filter(|f| f.is_code && !f.is_generated)
    {
        if !is_streaming_checked_path(&file.rel_path) || streaming_adapter_path(ctx, &file.rel_path)
        {
            continue;
        }
        let lower = file.text.to_ascii_lowercase();
        let Some(marker) = STREAMING_CLIENT_PATTERNS
            .iter()
            .find(|marker| lower.contains(**marker))
        else {
            continue;
        };
        if marker.contains("kafka") && kafka_exception_with_migration_path(ctx) {
            continue;
        }
        hits.push(FindingHit {
            path: file.rel_path.clone(),
            line: first_line_containing(&file.text, marker),
            text: (*marker).into(),
            matched_term: Some((*marker).into()),
            agent_fix: "move streaming clients behind the queue adapter boundary or document a brownfield exception with a migration path".into(),
            problem: format!(
                "streaming client marker `{marker}` appears outside `crates/adapters/queues`"
            ),
        });
        if hits.len() >= 20 {
            break;
        }
    }
    hits
}

fn is_streaming_checked_path(path: &str) -> bool {
    !(path.starts_with("docs/")
        || path.starts_with("paper/")
        || path.starts_with("reference/")
        || path.starts_with("tips/")
        || path.starts_with("agent/")
        || path.starts_with(".github/")
        || path.starts_with("packages/ux-qa/")
        || path.starts_with("crates/jankurai/"))
}

fn streaming_adapter_path(ctx: &AuditContext, path: &str) -> bool {
    if path.starts_with("crates/adapters/queues/")
        || path.starts_with("crates/adapters/src/queues/")
        || path.starts_with("adapters/queues/")
        || path.starts_with("apps/api/src/adapters/queues/")
    {
        return true;
    }
    boundary_manifest(ctx)
        .and_then(|manifest| manifest.queues)
        .map(|queues| {
            queues
                .adapter_paths
                .iter()
                .any(|adapter_path| path_matches_prefix(path, adapter_path))
        })
        .unwrap_or(false)
}

fn kafka_exception_with_migration_path(ctx: &AuditContext) -> bool {
    boundary_manifest(ctx)
        .map(|manifest| {
            manifest.streaming_exception.iter().any(|exception| {
                exception.runtime.eq_ignore_ascii_case("kafka")
                    && exception
                        .classification
                        .as_deref()
                        .or(exception.reason.as_deref())
                        .map(|value| value.eq_ignore_ascii_case("brownfield"))
                        .unwrap_or(false)
                    && !exception.owner.trim().is_empty()
                    && !exception.migration_path.trim().is_empty()
            })
        })
        .unwrap_or(false)
}

fn first_line_containing(text: &str, needle: &str) -> Option<usize> {
    let needle = needle.to_ascii_lowercase();
    text.lines()
        .position(|line| line.to_ascii_lowercase().contains(&needle))
        .map(|index| index + 1)
}

const GENERATED_ZONES_MANIFEST: &str = "agent/generated-zones.toml";

/// Returns findings when `agent/generated-zones.toml` exists, parses as zone rows, and any
/// `[[zone]]` omits non-empty `path`, `source`, or `command` (Phase 07 / generated-zone reproducibility).
pub fn generated_zone_manifest_metadata_issues(ctx: &AuditContext) -> Vec<FindingHit> {
    let full = ctx.root.join(GENERATED_ZONES_MANIFEST);
    if !full.exists() {
        return vec![];
    }
    let text = match std::fs::read_to_string(&full) {
        Ok(t) => t,
        Err(_) => return vec![],
    };
    let file: crate::commands::context_data::GeneratedZonesFile = match toml::from_str(&text) {
        Ok(f) => f,
        Err(_) => return vec![],
    };
    let mut problems: Vec<String> = vec![];
    for (index, zone) in file.zone.iter().enumerate() {
        let path = zone.path.trim();
        let source = zone.source.trim();
        let command = zone.command.trim();
        if path.is_empty() {
            problems.push(format!("zone[{index}]: missing or empty `path`"));
            continue;
        }
        if source.is_empty() {
            problems.push(format!(
                "zone `{}`: missing or empty `source`",
                zone.path.trim()
            ));
        }
        if command.is_empty() {
            problems.push(format!(
                "zone `{}`: missing or empty `command`",
                zone.path.trim()
            ));
        }
    }
    if problems.is_empty() {
        return vec![];
    }
    let detail = problems.join("; ");
    vec![FindingHit::new(
        GENERATED_ZONES_MANIFEST,
        1,
        &format!("generated zone manifest has incomplete reproducibility metadata: {detail}"),
    )]
}

pub fn generated_zone_issues(ctx: &AuditContext) -> Vec<FindingHit> {
    let generated = ctx
        .all_files
        .iter()
        .filter(|f| f.is_generated && f.is_code)
        .cloned()
        .collect::<Vec<_>>();
    if generated.is_empty() {
        return vec![];
    }
    let mut issues = vec![];
    if !ctx
        .all_files
        .iter()
        .any(|f| f.rel_path == GENERATED_ZONES_MANIFEST)
    {
        issues.push(FindingHit::new(
            &generated[0].rel_path,
            1,
            "generated code exists without `agent/generated-zones.toml` ownership rules",
        ));
    }
    for file in generated {
        if !file.text.to_ascii_lowercase().contains("generated")
            && !file.text.to_ascii_lowercase().contains("do not edit")
        {
            issues.push(FindingHit::new(
                &file.rel_path,
                1,
                "generated file lacks a clear generated/do-not-edit marker",
            ));
        }
        if !pattern_hits(&vec![file.clone()], TODO_PATTERNS).is_empty() {
            issues.push(FindingHit::new(
                &file.rel_path,
                1,
                "generated file contains TODO/stub markers",
            ));
        }
    }
    issues
}

pub fn wrong_layer_db_hits(ctx: &AuditContext) -> Vec<FindingHit> {
    let mut hits = vec![];
    for file in product_files(ctx) {
        if !file.is_code && file.suffix != ".sql" {
            continue;
        }
        if file.rel_path.starts_with("db/")
            || file.rel_path.starts_with("migrations/")
            || file.rel_path.starts_with("crates/adapters/")
        {
            continue;
        }
        if ["apps/web/", "crates/domain/", "frontend/", "ui/", "src/"]
            .iter()
            .any(|p| file.rel_path.starts_with(p))
            && [
                "select ", "insert ", "update ", "delete ", "sqlx", "diesel", "psycopg", "sqlite3",
            ]
            .iter()
            .any(|m| file.text.to_ascii_lowercase().contains(m))
        {
            hits.push(FindingHit::new(
                &file.rel_path,
                1,
                "DB marker in non-adapter layer",
            ));
        }
    }
    hits
}

pub fn duplicate_blocks(ctx: &AuditContext) -> Vec<FindingHit> {
    let mut seen = std::collections::HashMap::new();
    let mut dups = vec![];
    for file in product_code_files(ctx) {
        let lines: Vec<_> = file
            .text
            .lines()
            .filter_map(|l| {
                let l = l.trim();
                if l.is_empty()
                    || l.starts_with("//")
                    || l.starts_with("#")
                    || l.starts_with("use ")
                    || l.starts_with("import ")
                {
                    return None;
                }
                let norm = l
                    .replace('"', "\"S\"")
                    .replace('\'', "\"S\"")
                    .replace('`', "\"S\"")
                    .chars()
                    .map(|c| if c.is_ascii_digit() { 'N' } else { c })
                    .collect::<String>();
                if norm.len() < 12 {
                    None
                } else {
                    Some(norm)
                }
            })
            .collect();
        for win in lines.windows(8) {
            let body = win.join("\n");
            if let Some(prev) = seen.insert(body.clone(), format!("{}:1", file.rel_path)) {
                dups.push(FindingHit::new(
                    &file.rel_path,
                    1,
                    &format!("duplicate block also appears at {}", prev),
                ));
            }
        }
    }
    dups
}

pub fn future_hostile_hits(ctx: &AuditContext) -> Vec<FindingHit> {
    let mut out = vec![];
    for file in product_code_files(ctx) {
        if is_future_hostile_allowlisted(&file) {
            continue;
        }
        for (idx, line) in file.text.lines().enumerate() {
            let lower = line.to_ascii_lowercase();
            if let Some(term) = FUTURE_HOSTILE_TERMS
                .iter()
                .find(|term| lower.contains(*term))
            {
                out.push(FindingHit { path: file.rel_path.clone(), line: Some(idx + 1), text: line.to_string(), matched_term: Some((*term).into()), agent_fix: "remove or rename the marker, implement the intended behavior, model a typed unsupported state, or move docs/generated/vendor/product-copy text into an allowlisted context".into(), problem: format!("future-hostile/dead-language term `{}` appears", term) });
            }
        }
    }
    out
}

fn is_future_hostile_allowlisted(file: &FileInfo) -> bool {
    file.is_generated
        || FUTURE_HOSTILE_ALLOWLIST_PREFIXES
            .iter()
            .any(|p| file.rel_path.starts_with(p))
        || FUTURE_HOSTILE_PRODUCT_COPY_PARTS
            .iter()
            .any(|p| file.rel_path.to_ascii_lowercase().contains(p))
}

pub fn manifest_parse_findings(ctx: &AuditContext) -> Vec<FindingHit> {
    use serde::Deserialize;
    use std::collections::HashMap;

    #[derive(Deserialize)]
    struct OwnerMapFile {
        #[allow(dead_code)]
        owners: HashMap<String, String>,
    }

    #[derive(Deserialize)]
    struct TestMapFile {
        #[allow(dead_code)]
        tests: HashMap<String, TestMapEntry>,
    }

    #[derive(Deserialize)]
    struct TestMapEntry {
        #[allow(dead_code)]
        command: String,
        #[allow(dead_code)]
        purpose: Option<String>,
    }

    let mut out = Vec::new();
    let json_manifests: &[(&str, fn(&str) -> std::result::Result<(), String>)] = &[
        ("agent/owner-map.json", |text| {
            serde_json::from_str::<OwnerMapFile>(text)
                .map(|_| ())
                .map_err(|err| err.to_string())
        }),
        ("agent/test-map.json", |text| {
            serde_json::from_str::<TestMapFile>(text)
                .map(|_| ())
                .map_err(|err| err.to_string())
        }),
    ];
    for (path, parse) in json_manifests {
        let full = ctx.root.join(path);
        if !full.exists() {
            continue;
        }
        let text = std::fs::read_to_string(&full).unwrap_or_default();
        if let Err(err) = parse(&text) {
            out.push(FindingHit::new(path, 1, &err));
        }
    }
    for path in [
        "agent/generated-zones.toml",
        "agent/boundaries.toml",
        "agent/proof-lanes.toml",
        "agent/standard-version.toml",
        "agent/audit-policy.toml",
    ] {
        let full = ctx.root.join(path);
        if !full.exists() {
            continue;
        }
        let text = std::fs::read_to_string(&full).unwrap_or_default();
        if let Err(err) = toml::from_str::<toml::Value>(&text) {
            out.push(FindingHit::new(path, 1, &err.to_string()));
        }
    }
    out
}

pub fn ci_hardening_hits(ctx: &AuditContext) -> Vec<FindingHit> {
    let mut hits = vec![];
    for file in ctx.all_files.iter().filter(|f| {
        !f.is_generated
            && f.rel_path.starts_with(".github/workflows/")
            && (f.rel_path.ends_with(".yml") || f.rel_path.ends_with(".yaml"))
    }) {
        for (idx, line) in file.text.lines().enumerate() {
            let line_lower = line.to_ascii_lowercase();
            if line_lower.contains("permissions:") && line_lower.contains("write-all") {
                hits.push(FindingHit {
                    path: file.rel_path.clone(),
                    line: Some(idx + 1),
                    text: line.to_string(),
                    matched_term: Some("write-all".into()),
                    agent_fix: "replace write-all permissions with explicit minimum required scopes (e.g. contents: read, actions: read)".into(),
                    problem: "workflow declares broad write-all permissions".into(),
                });
            }
            if line_lower.contains("run:")
                && line_lower.contains("echo ")
                && line_lower.contains("${{")
                && line_lower.contains("secrets.")
            {
                hits.push(FindingHit {
                    path: file.rel_path.clone(),
                    line: Some(idx + 1),
                    text: line.to_string(),
                    matched_term: Some("echo secrets".into()),
                    agent_fix: "never echo secrets; mask them or use environment variables passed directly to trusted binaries".into(),
                    problem: "workflow potentially leaks secrets via echo".into(),
                });
            }
            if line_lower.contains("uses:") && line_lower.contains("@master") {
                hits.push(FindingHit {
                    path: file.rel_path.clone(),
                    line: Some(idx + 1),
                    text: line.to_string(),
                    matched_term: Some("@master".into()),
                    agent_fix: "pin action to a specific commit SHA or stable semver tag".into(),
                    problem: "workflow uses unpinned @master action".into(),
                });
            }
            if hits.len() >= 20 {
                return hits;
            }
        }
    }
    hits
}
