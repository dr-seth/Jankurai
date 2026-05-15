//! `jankurai ai audit` engine.
//!
//! Scans a directory tree for AI/LLM SDK call sites and classifies each
//! into one of four replaceability tiers. Classification re-derives from
//! import-parse + call-shape inspection (Rule 9) — it never tier-classifies
//! by regex-matching an SDK package name against file content. The
//! `oauth_falsepositive` / `llm_settings_falsepositive` fixtures prove the
//! distinction: an SDK name in a string literal or config key, with no
//! module-level import, is Tier 4 FALSE_POSITIVE.

use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;

use super::AiAuditArgs;

// ---------------------------------------------------------------------------
// DEDUPE: the provider table + source-scan helpers below are copied verbatim
// from `commands/migrate/prompt_verify.rs`. They are private there and verb
// #1's enhancements live in un-merged PRs #4/#5, so extracting a shared
// module now would conflict. Fold into a shared `ai_common` once #4/#5 land.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
struct LlmProvider {
    name: &'static str,
    import_patterns: &'static [&'static str],
    invocation_patterns: &'static [&'static str],
}

const SUPPORTED_LLM_PROVIDERS: &[LlmProvider] = &[
    LlmProvider {
        name: "openai",
        import_patterns: &[
            r"(?m)^\s*use\s+openai::",
            r"(?m)^\s*import\s+openai\b",
            r"(?m)^\s*from\s+openai\s+import\s+OpenAI\b",
            r#"(?m)^\s*import\s+.*\bOpenAI\b.*\bfrom\s+["']openai["']"#,
            r#"(?m)^\s*(const|let|var)\s+\{?\s*OpenAI\s*\}?\s*=\s*require\(["']openai["']\)"#,
        ],
        invocation_patterns: &[
            r"\.responses\.create\s*\(",
            r"\.chat\.completions\.create\s*\(",
            r"\.completions\.create\s*\(",
        ],
    },
    LlmProvider {
        name: "anthropic",
        import_patterns: &[
            r"(?m)^\s*import\s+anthropic\b",
            r"(?m)^\s*from\s+anthropic\s+import\s+Anthropic\b",
            r"(?m)^\s*use\s+anthropic::",
            r#"(?m)^\s*import\s+.*\bAnthropic\b.*\bfrom\s+["'](@anthropic-ai/sdk|anthropic)["']"#,
            r#"(?m)^\s*(const|let|var)\s+\{?\s*Anthropic\s*\}?\s*=\s*require\(["'](@anthropic-ai/sdk|anthropic)["']\)"#,
        ],
        invocation_patterns: &[r"\.messages\.create\s*\(", r"\.completions\.create\s*\("],
    },
    LlmProvider {
        name: "langchain",
        import_patterns: &[
            r"(?m)^\s*(from\s+langchain|import\s+langchain|use\s+langchain::)",
            r#"(?m)^\s*import\s+.*\bfrom\s+["'](@?langchain/[^"']+|langchain)["']"#,
            r#"(?m)^\s*(const|let|var)\s+.*=\s*require\(["'](@?langchain/[^"']+|langchain)["']\)"#,
        ],
        invocation_patterns: &[r"\.invoke\s*\(", r"\.predict\s*\(", r"\.generate\s*\("],
    },
    LlmProvider {
        name: "llamaindex",
        import_patterns: &[
            r"(?m)^\s*(from\s+llama_index|import\s+llama_index|use\s+llama_index::)",
            r#"(?m)^\s*import\s+.*\bfrom\s+["'](llamaindex|llama_index)["']"#,
            r#"(?m)^\s*(const|let|var)\s+.*=\s*require\(["'](llamaindex|llama_index)["']\)"#,
        ],
        invocation_patterns: &[r"\.query\s*\(", r"\.chat\s*\(", r"\.complete\s*\("],
    },
];

/// Bare provider-name tokens used only for the FALSE_POSITIVE heuristic:
/// the name appears in source but no `import_patterns` entry matched.
const PROVIDER_NAME_TOKENS: &[&str] = &["anthropic", "openai", "langchain", "llama_index"];

fn regex(pattern: &str) -> Regex {
    Regex::new(pattern).expect("valid regex")
}

fn is_comment_only(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return true;
    }
    let prefixes = ["//", "#", "/*", "*", "--", ";", "%", "<!--"];
    prefixes.iter().any(|prefix| trimmed.starts_with(prefix))
}

fn strip_obvious_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//")
            || trimmed.starts_with('#')
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
        {
            out.push('\n');
            continue;
        }
        let without_comment = line.split_once("//").map(|(code, _)| code).unwrap_or(line);
        out.push_str(without_comment);
        out.push('\n');
    }
    out
}

fn enclosing_scope_ok(text: &str, invocation_line_idx: usize) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    let line = lines.get(invocation_line_idx).copied().unwrap_or("");
    let indent = line.chars().take_while(|c| c.is_whitespace()).count();
    if indent == 0 {
        return false;
    }
    for prev in lines[..invocation_line_idx].iter().rev() {
        let trimmed = prev.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }
        let prev_indent = prev.chars().take_while(|c| c.is_whitespace()).count();
        if prev_indent < indent
            && (trimmed.starts_with("def ")
                || trimmed.starts_with("async def ")
                || trimmed.starts_with("class ")
                || trimmed.starts_with("fn ")
                || trimmed.starts_with("async fn ")
                || trimmed.starts_with("impl ")
                || trimmed.starts_with("function ")
                || trimmed.starts_with("export function ")
                || trimmed.starts_with("const ")
                || trimmed.starts_with("let "))
        {
            return true;
        }
    }
    false
}

fn is_skipped_dir(path: &Path) -> bool {
    path.components().any(|component| match component {
        Component::Normal(part) => matches!(
            part.to_string_lossy().as_ref(),
            ".git" | "target" | "node_modules" | "dist" | "build"
        ),
        _ => false,
    })
}

fn is_candidate_source(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default(),
        "py" | "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" | "rs"
    )
}

fn candidate_source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| !entry.file_type().is_dir() || !is_skipped_dir(entry.path()))
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if entry.file_type().is_dir() || !is_candidate_source(path) {
            continue;
        }
        files.push(path.to_path_buf());
    }
    files.sort();
    files
}

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Tier {
    Tier1,
    Tier2,
    Tier3,
    Tier4,
}

impl Tier {
    fn label(self) -> &'static str {
        match self {
            Tier::Tier1 => "tier_1",
            Tier::Tier2 => "tier_2",
            Tier::Tier3 => "tier_3",
            Tier::Tier4 => "tier_4",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SiteShape {
    Integrated,
    Classify,
    Wrapper,
    Generate,
    Agent,
    FalsePositive,
    NoCallSite,
}

impl SiteShape {
    fn label(self) -> &'static str {
        match self {
            SiteShape::Integrated => "INTEGRATED",
            SiteShape::Classify => "CLASSIFY",
            SiteShape::Wrapper => "WRAPPER",
            SiteShape::Generate => "GENERATE",
            SiteShape::Agent => "AGENT",
            SiteShape::FalsePositive => "FALSE_POSITIVE",
            SiteShape::NoCallSite => "no_call_site",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SiteRecord {
    pub file: String,
    pub line: usize,
    pub shape: String,
    /// `None` for `no_call_site` (excluded from the tier summary counts).
    pub tier: Option<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditReport {
    pub schema_version: String,
    pub command: String,
    pub sites_total: usize,
    pub tier1_files: usize,
    pub tier2_files: usize,
    pub tier3_files: usize,
    pub tier4_files: usize,
    pub tier2_sites: usize,
    pub sites: Vec<SiteRecord>,
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct AuditConfig {
    classify_max_tokens_max: u32,
    classify_temperature_max: f64,
    wrapper_enumerated_options_min: usize,
    generate_max_tokens_min: u32,
    nano_bridge_modules: Vec<String>,
    glob_overrides: Vec<String>,
    manual: Vec<(String, usize, Tier)>,
}

impl Default for AuditConfig {
    fn default() -> Self {
        AuditConfig {
            classify_max_tokens_max: 500,
            classify_temperature_max: 0.3,
            wrapper_enumerated_options_min: 4,
            generate_max_tokens_min: 1000,
            nano_bridge_modules: vec![
                "arya_speaks.language_core".to_string(),
                "arya_speaks.language_core.llm_adapter".to_string(),
            ],
            glob_overrides: vec!["**/oauth.py".to_string(), "**/llm_settings.py".to_string()],
            manual: Vec::new(),
        }
    }
}

fn load_config(path: Option<&Path>) -> Result<AuditConfig> {
    let Some(path) = path else {
        return Ok(AuditConfig::default());
    };
    let text = fs::read_to_string(path)
        .with_context(|| format!("read ai-audit config {}", path.display()))?;
    let value: toml::Value =
        toml::from_str(&text).with_context(|| format!("parse TOML {}", path.display()))?;
    let mut cfg = AuditConfig::default();

    if let Some(h) = value.get("heuristics") {
        if let Some(v) = h
            .get("classify_max_tokens_max")
            .and_then(|v| v.as_integer())
        {
            cfg.classify_max_tokens_max = v as u32;
        }
        if let Some(v) = h.get("classify_temperature_max").and_then(|v| v.as_float()) {
            cfg.classify_temperature_max = v;
        }
        if let Some(v) = h
            .get("wrapper_enumerated_options_min")
            .and_then(|v| v.as_integer())
        {
            cfg.wrapper_enumerated_options_min = v as usize;
        }
        if let Some(v) = h
            .get("generate_max_tokens_min")
            .and_then(|v| v.as_integer())
        {
            cfg.generate_max_tokens_min = v as u32;
        }
    }
    if let Some(arr) = value
        .get("bridges")
        .and_then(|b| b.get("nano_bridge_modules"))
        .and_then(|v| v.as_array())
    {
        cfg.nano_bridge_modules = arr
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();
    }
    if let Some(arr) = value
        .get("false_positives")
        .and_then(|f| f.get("glob_overrides"))
        .and_then(|v| v.as_array())
    {
        cfg.glob_overrides = arr
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();
    }
    if let Some(arr) = value
        .get("overrides")
        .and_then(|o| o.get("manual"))
        .and_then(|v| v.as_array())
    {
        for entry in arr {
            if let Some(tuple) = entry.as_array() {
                if let (Some(file), Some(line), Some(tier)) = (
                    tuple.first().and_then(|v| v.as_str()),
                    tuple.get(1).and_then(|v| v.as_integer()),
                    tuple.get(2).and_then(|v| v.as_str()),
                ) {
                    let tier = match tier {
                        "tier_1" => Tier::Tier1,
                        "tier_2" => Tier::Tier2,
                        "tier_3" => Tier::Tier3,
                        _ => Tier::Tier4,
                    };
                    cfg.manual.push((file.to_string(), line as usize, tier));
                }
            }
        }
    }
    Ok(cfg)
}

/// Minimal `**/name.py`-style glob: only the trailing component matters.
fn glob_matches(pattern: &str, path: &Path) -> bool {
    let needle = pattern.trim_start_matches("**/");
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n == needle)
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Call-shape extraction
// ---------------------------------------------------------------------------

static TEMP_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"temperature\s*=\s*([0-9]*\.?[0-9]+)").expect("temp re"));
static MAXTOK_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"max_tokens\s*=\s*([0-9]+)").expect("maxtok re"));
static JOIN_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\.join\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)").expect("join re"));

/// Collect the call's argument text by walking forward from `start_idx`
/// until parentheses balance (handles multi-line calls).
fn call_arg_blob(lines: &[&str], start_idx: usize) -> String {
    let mut depth: i32 = 0;
    let mut started = false;
    let mut blob = String::new();
    for line in &lines[start_idx..] {
        for ch in line.chars() {
            if ch == '(' {
                depth += 1;
                started = true;
            } else if ch == ')' {
                depth -= 1;
            }
        }
        blob.push_str(line);
        blob.push('\n');
        if started && depth <= 0 {
            break;
        }
    }
    blob
}

/// Count the elements of a module-level `NAME = [ ... ]` list literal by
/// counting top-level string elements. Returns 0 if not found.
fn list_literal_len(code: &str, name: &str) -> usize {
    let assign = Regex::new(&format!(
        r"(?ms)^\s*{}\s*=\s*\[(.*?)\]",
        regex::escape(name)
    ))
    .expect("list-literal re");
    let Some(cap) = assign.captures(code) else {
        return 0;
    };
    let body = cap.get(1).map(|m| m.as_str()).unwrap_or("");
    // Count quoted string elements (handles trailing comma + multi-line).
    let str_elem = Regex::new(r#"["'][^"']*["']"#).expect("str-elem re");
    str_elem.find_iter(body).count()
}

struct CallShape {
    temperature: Option<f64>,
    max_tokens: Option<u32>,
    has_tools: bool,
    has_response_format: bool,
    enum_options: usize,
    json_shape_instruction: bool,
}

fn analyze_call(code: &str, lines: &[&str], call_idx: usize) -> CallShape {
    let blob = call_arg_blob(lines, call_idx);
    let temperature = TEMP_RE
        .captures(&blob)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<f64>().ok());
    let max_tokens = MAXTOK_RE
        .captures(&blob)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<u32>().ok());
    let has_tools = Regex::new(r"\btools\s*=").unwrap().is_match(&blob);
    let has_response_format = blob.contains("response_format");

    // Closed-set enum: a `.join(NAME)` whose NAME is a module-level list
    // literal. Count the literal's elements.
    let mut enum_options = 0usize;
    for cap in JOIN_RE.captures_iter(code) {
        if let Some(name) = cap.get(1) {
            let n = list_literal_len(code, name.as_str());
            if n > enum_options {
                enum_options = n;
            }
        }
    }

    // JSON-shape instruction anywhere in the file's system-prompt prose.
    let lc = code.to_ascii_lowercase();
    let json_shape_instruction = lc.contains("json object")
        || lc.contains("respond with a json")
        || (code.contains("{\"") && code.contains("\"}"))
        || lc.contains("of shape {");

    CallShape {
        temperature,
        max_tokens,
        has_tools,
        has_response_format,
        enum_options,
        json_shape_instruction,
    }
}

// ---------------------------------------------------------------------------
// Classification
// ---------------------------------------------------------------------------

fn classify(shape: &CallShape, cfg: &AuditConfig) -> (SiteShape, Tier, String) {
    if shape.has_tools {
        return (
            SiteShape::Agent,
            Tier::Tier3,
            "tool-use loop (tools= kwarg) → permanent agent".to_string(),
        );
    }

    let max_tokens = shape.max_tokens.unwrap_or(0);
    let temperature = shape.temperature.unwrap_or(1.0);

    let classify_ok = temperature <= cfg.classify_temperature_max
        && max_tokens <= cfg.classify_max_tokens_max
        && shape.enum_options >= 2
        && (shape.json_shape_instruction || shape.has_response_format);
    if classify_ok {
        return (
            SiteShape::Classify,
            Tier::Tier2,
            format!(
                "temp={temperature} max_tokens={max_tokens} closed-set enum({}) + JSON-shape → nano-replaceable",
                shape.enum_options
            ),
        );
    }

    if shape.enum_options >= cfg.wrapper_enumerated_options_min {
        return (
            SiteShape::Wrapper,
            Tier::Tier2,
            format!(
                "system prompt enumerates {} options → classification dressed as free-form",
                shape.enum_options
            ),
        );
    }

    if max_tokens >= cfg.generate_max_tokens_min {
        return (
            SiteShape::Generate,
            Tier::Tier3,
            format!("max_tokens={max_tokens} free-form → permanent LLM"),
        );
    }

    (
        SiteShape::Generate,
        Tier::Tier3,
        format!("max_tokens={max_tokens} no closed-set enum → treat as permanent"),
    )
}

// ---------------------------------------------------------------------------
// Per-file audit
// ---------------------------------------------------------------------------

fn imports_provider(code: &str, provider: &LlmProvider) -> bool {
    provider
        .import_patterns
        .iter()
        .any(|p| regex(p).is_match(code))
}

fn imports_nano_bridge(code: &str, modules: &[String]) -> Option<String> {
    for m in modules {
        let re = Regex::new(&format!(
            r"(?m)^\s*(?:from|import)\s+{}\b",
            regex::escape(m)
        ))
        .expect("bridge re");
        if re.is_match(code) {
            return Some(m.clone());
        }
    }
    None
}

fn first_match_line(text: &str, pattern: &Regex) -> usize {
    for (idx, line) in text.lines().enumerate() {
        if pattern.is_match(line) {
            return idx + 1;
        }
    }
    1
}

fn audit_file(path: &Path, rel: &str, cfg: &AuditConfig) -> Vec<SiteRecord> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let code = strip_obvious_comments(&text);
    let lines: Vec<&str> = code.lines().collect();

    // 1. Tier 1 INTEGRATED — module routes through the project nano-bridge.
    if let Some(bridge) = imports_nano_bridge(&code, &cfg.nano_bridge_modules) {
        let line = first_match_line(
            &code,
            &Regex::new(&format!(
                r"^\s*(?:from|import)\s+{}",
                regex::escape(&bridge)
            ))
            .unwrap(),
        );
        return vec![SiteRecord {
            file: rel.to_string(),
            line,
            shape: SiteShape::Integrated.label().to_string(),
            tier: Some(Tier::Tier1.label().to_string()),
            notes: format!("imports nano-bridge `{bridge}` → already nano-routed; skip"),
        }];
    }

    // 2. Which real SDK providers are imported at module level?
    let imported: Vec<&LlmProvider> = SUPPORTED_LLM_PROVIDERS
        .iter()
        .filter(|p| imports_provider(&code, p))
        .collect();

    // 3. No provider import. Rule 9: an SDK name present only as a string
    //    literal / config key (or a glob-override file) is FALSE_POSITIVE,
    //    NOT a call site.
    if imported.is_empty() {
        let name_token = PROVIDER_NAME_TOKENS.iter().find(|t| text.contains(**t));
        let glob_hit = cfg.glob_overrides.iter().any(|g| glob_matches(g, path));
        if name_token.is_some() || glob_hit {
            let token = name_token.copied().unwrap_or("sdk");
            let token_re = Regex::new(&format!(r#"["']?{}"#, regex::escape(token))).unwrap();
            let line = first_match_line(&text, &token_re);
            return vec![SiteRecord {
                file: rel.to_string(),
                line,
                shape: SiteShape::FalsePositive.label().to_string(),
                tier: Some(Tier::Tier4.label().to_string()),
                notes: format!(
                    "`{token}` present only as string/identifier; no module import → false positive"
                ),
            }];
        }
        return Vec::new();
    }

    // 4. Provider(s) imported — find invocation sites.
    let mut records = Vec::new();
    for provider in &imported {
        for (idx, line) in lines.iter().enumerate() {
            if is_comment_only(line) {
                continue;
            }
            let hit = provider
                .invocation_patterns
                .iter()
                .any(|p| regex(p).is_match(line));
            if !hit || !enclosing_scope_ok(&code, idx) {
                continue;
            }
            let shape = analyze_call(&code, &lines, idx);
            let (site_shape, mut tier, notes) = classify(&shape, cfg);

            // Manual per-site override (config escape hatch).
            if let Some((_, _, forced)) = cfg
                .manual
                .iter()
                .find(|(f, l, _)| rel.ends_with(f.as_str()) && *l == idx + 1)
            {
                tier = *forced;
            }

            records.push(SiteRecord {
                file: rel.to_string(),
                line: idx + 1,
                shape: site_shape.label().to_string(),
                tier: Some(tier.label().to_string()),
                notes: format!("{} [{}]", notes, provider.name),
            });
        }
    }

    // Provider imported but never invoked → dangling import (Rule 9 case 3).
    if records.is_empty() {
        records.push(SiteRecord {
            file: rel.to_string(),
            line: 0,
            shape: SiteShape::NoCallSite.label().to_string(),
            tier: None,
            notes: "SDK import present but no invocation call site".to_string(),
        });
    }
    records
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(args: AiAuditArgs) -> Result<()> {
    let cfg = load_config(args.config.as_deref())?;
    let mut sites: Vec<SiteRecord> = Vec::new();
    let mut seen_files: BTreeSet<String> = BTreeSet::new();

    for dir in &args.dirs {
        let base =
            fs::canonicalize(dir).with_context(|| format!("canonicalize {}", dir.display()))?;
        for path in candidate_source_files(&base) {
            let rel = path
                .strip_prefix(&base)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            if !seen_files.insert(rel.clone()) {
                continue;
            }
            sites.extend(audit_file(&path, &rel, &cfg));
        }
    }

    sites.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));

    let files_in_tier = |t: &str| -> usize {
        sites
            .iter()
            .filter(|s| s.tier.as_deref() == Some(t))
            .map(|s| s.file.clone())
            .collect::<BTreeSet<_>>()
            .len()
    };
    let tier1 = files_in_tier("tier_1");
    let tier2 = files_in_tier("tier_2");
    let tier3 = files_in_tier("tier_3");
    let tier4 = files_in_tier("tier_4");
    let tier2_sites = sites
        .iter()
        .filter(|s| s.tier.as_deref() == Some("tier_2"))
        .count();

    let report = AuditReport {
        schema_version: "1.0.0".to_string(),
        command: "jankurai ai audit".to_string(),
        sites_total: sites.len(),
        tier1_files: tier1,
        tier2_files: tier2,
        tier3_files: tier3,
        tier4_files: tier4,
        tier2_sites,
        sites: sites.clone(),
    };

    // Per-site table. `no_call_site` placeholders are not real sites.
    let real_sites = sites.iter().filter(|s| s.tier.is_some()).count();
    println!("{real_sites} LLM call sites detected");
    println!("Per-site classification:");
    println!();
    println!("| File | Line | Site shape | Tier | Notes |");
    println!("|------|------|-----------|------|-------|");
    for s in &sites {
        println!(
            "| {} | {} | {} | {} | {} |",
            s.file,
            s.line,
            s.shape,
            s.tier.as_deref().unwrap_or("n/a"),
            s.notes
        );
    }
    println!();
    println!("Replaceability summary:");
    println!(
        "  Tier 1 (already integrated):  {} {}",
        tier1,
        pluralize(tier1)
    );
    println!(
        "  Tier 2 (replaceable):      {} {}, ~{} sites",
        tier2,
        pluralize(tier2),
        tier2_sites
    );
    println!(
        "  Tier 3 (permanent):        {} {}",
        tier3,
        pluralize(tier3)
    );
    println!(
        "  Tier 4 (false positive):      {} {}",
        tier4,
        pluralize(tier4)
    );

    if let Some(path) = args.out.as_deref() {
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(path, serde_json::to_string_pretty(&report)?)
            .with_context(|| format!("write {path}"))?;
    }
    if let Some(path) = args.md.as_deref() {
        crate::render::write_markdown(path, &render_markdown(&report))?;
    }

    Ok(())
}

fn pluralize(n: usize) -> &'static str {
    if n == 1 {
        "file"
    } else {
        "files"
    }
}

fn render_markdown(report: &AuditReport) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let _ = writeln!(out, "# jankurai ai audit");
    let _ = writeln!(out);
    let _ = writeln!(out, "- sites total: `{}`", report.sites_total);
    let _ = writeln!(out, "- tier 1 files: `{}`", report.tier1_files);
    let _ = writeln!(out, "- tier 2 files: `{}`", report.tier2_files);
    let _ = writeln!(out, "- tier 3 files: `{}`", report.tier3_files);
    let _ = writeln!(out, "- tier 4 files: `{}`", report.tier4_files);
    let _ = writeln!(out);
    let _ = writeln!(out, "| File | Line | Shape | Tier | Notes |");
    let _ = writeln!(out, "|------|------|-------|------|-------|");
    for s in &report.sites {
        let _ = writeln!(
            out,
            "| `{}` | {} | {} | {} | {} |",
            s.file,
            s.line,
            s.shape,
            s.tier.as_deref().unwrap_or("n/a"),
            s.notes
        );
    }
    out
}
