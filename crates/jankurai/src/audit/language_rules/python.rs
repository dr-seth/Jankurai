use super::catalog::{
    ConfidencePolicy, Language, LanguageFinding, LanguageRule, Matcher, ProofWindow,
};
use crate::audit::helpers::AuditContext;
use crate::model::FileInfo;

const HLT_RULE_ID: &str = "HLT-033-PYTHON-BAD-BEHAVIOR";
const DETECTOR_DYNAMIC_CODE: &str = "python.exec.dynamic-code";
const DETECTOR_UNSAFE_DESER: &str = "python.deser.unsafe-object";
const DETECTOR_SHELL_DYNAMIC: &str = "python.shell.dynamic";
const DETECTOR_SQL_STRING_BUILT: &str = "python.sql.string-built";
const DETECTOR_TLS_DEBUG: &str = "python.net.tls-debug";
const DETECTOR_BROAD_EXCEPT: &str = "python.review.broad-except";

const RULES: &[LanguageRule] = &[
    LanguageRule {
        id: DETECTOR_DYNAMIC_CODE,
        language: Language::Python,
        hlt_rule_id: HLT_RULE_ID,
        severity: "high",
        category: "security",
        lane: "contract",
        confidence: ConfidencePolicy::High,
        matcher: Matcher::ContainsAny(&["eval(", "exec(", "compile(", "exec(open("]),
        proof_window: ProofWindow::None,
        problem: "dynamic code execution runs attacker-controlled source directly",
        fix: "replace dynamic execution with a parser, dispatch table, or typed plugin boundary",
    },
    LanguageRule {
        id: DETECTOR_UNSAFE_DESER,
        language: Language::Python,
        hlt_rule_id: HLT_RULE_ID,
        severity: "high",
        category: "security",
        lane: "contract",
        confidence: ConfidencePolicy::High,
        matcher: Matcher::ContainsAny(&[
            "pickle.load",
            "pickle.loads",
            "dill.load",
            "dill.loads",
            "cloudpickle.load",
            "cloudpickle.loads",
            "marshal.load",
            "marshal.loads",
            "shelve.open",
            "joblib.load",
            "yaml.load(",
        ]),
        proof_window: ProofWindow::None,
        problem: "unsafe deserialisation can instantiate attacker-controlled objects",
        fix: "use `safe_load` or a schema-based decoder for untrusted input",
    },
    LanguageRule {
        id: DETECTOR_SHELL_DYNAMIC,
        language: Language::Python,
        hlt_rule_id: HLT_RULE_ID,
        severity: "high",
        category: "security",
        lane: "contract",
        confidence: ConfidencePolicy::High,
        matcher: Matcher::ContainsAny(&["os.system(", "os.popen(", "shell=True"]),
        proof_window: ProofWindow::None,
        problem: "shell=True or os.system lets input reach a shell",
        fix: "pass argv arrays to subprocess and keep shell off",
    },
    LanguageRule {
        id: DETECTOR_SQL_STRING_BUILT,
        language: Language::Python,
        hlt_rule_id: HLT_RULE_ID,
        severity: "high",
        category: "security",
        lane: "contract",
        confidence: ConfidencePolicy::High,
        matcher: Matcher::ContainsAny(&["execute(", "executemany(", "query("]),
        proof_window: ProofWindow::None,
        problem: "string-built SQL reaches a database sink without parameter binding",
        fix: "parameterize the statement or move identifier handling through an allowlist",
    },
    LanguageRule {
        id: DETECTOR_TLS_DEBUG,
        language: Language::Python,
        hlt_rule_id: HLT_RULE_ID,
        severity: "high",
        category: "security",
        lane: "contract",
        confidence: ConfidencePolicy::High,
        matcher: Matcher::ContainsAny(&[
            "verify=False",
            "disable_warnings",
            "_create_unverified_context",
        ]),
        proof_window: ProofWindow::None,
        problem: "certificate verification is disabled in runtime code",
        fix: "remove the debug bypass and pin a trusted CA bundle",
    },
    LanguageRule {
        id: DETECTOR_BROAD_EXCEPT,
        language: Language::Python,
        hlt_rule_id: HLT_RULE_ID,
        severity: "medium",
        category: "advisory",
        lane: "contract",
        confidence: ConfidencePolicy::Low,
        matcher: Matcher::ContainsAny(&["except exception", "except baseexception"]),
        proof_window: ProofWindow::None,
        problem: "broad exception handling hides real failures and control flow",
        fix: "catch specific exceptions and keep the failure surface explicit",
    },
];

#[derive(Debug, Clone, Copy, Default)]
pub struct PythonSummary {
    pub hard_findings: usize,
    pub advisory_signals: usize,
}

pub fn catalog() -> &'static [LanguageRule] {
    RULES
}

pub fn summary(ctx: &AuditContext) -> PythonSummary {
    PythonSummary {
        hard_findings: findings(ctx).len(),
        advisory_signals: advisory_signals(ctx).len(),
    }
}

pub fn findings(ctx: &AuditContext) -> Vec<LanguageFinding> {
    let mut out = hard_findings(ctx);
    out.sort_by(sort_key);
    out
}

pub fn advisory_signals(ctx: &AuditContext) -> Vec<LanguageFinding> {
    let mut out = Vec::new();
    for file in python_files(ctx) {
        for (idx, line) in file.text.lines().enumerate() {
            if let Some(hit) = advisory_hit_for_line(&file, idx + 1, line) {
                out.push(hit);
            }
        }
    }
    out.sort_by(sort_key);
    out
}

fn hard_findings(ctx: &AuditContext) -> Vec<LanguageFinding> {
    let mut out = Vec::new();
    for file in python_files(ctx) {
        for (idx, line) in file.text.lines().enumerate() {
            if let Some(hit) = hard_hit_for_line(&file, idx + 1, line) {
                out.push(hit);
            }
        }
    }
    out
}

fn python_files(ctx: &AuditContext) -> Vec<FileInfo> {
    ctx.all_files
        .iter()
        .filter(|file| is_python_candidate(file))
        .cloned()
        .collect()
}

fn is_python_candidate(file: &FileInfo) -> bool {
    let rel = file.rel_path.to_ascii_lowercase();
    !file.is_generated && !is_excluded_path(&rel) && matches!(file.suffix.as_str(), ".py" | ".pyi")
}

fn is_excluded_path(rel: &str) -> bool {
    rel.starts_with("docs/")
        || rel.starts_with("paper/")
        || rel.starts_with("reference/")
        || rel.starts_with("tips/")
        || rel.starts_with("target/")
        || rel.starts_with("tests/")
        || rel.contains("/tests/")
        || rel.starts_with("examples/")
        || rel.contains("/examples/")
        || rel.starts_with("generated/")
        || rel.contains("/generated/")
}

fn hard_hit_for_line(file: &FileInfo, line_no: usize, line: &str) -> Option<LanguageFinding> {
    let Some(normalized) = normalize_python_line(line) else {
        return None;
    };
    let lower = normalized.to_ascii_lowercase();
    if lower.is_empty() {
        return None;
    }

    if is_dynamic_code_line(&lower) {
        return Some(finding(
            DETECTOR_DYNAMIC_CODE,
            "eval",
            file,
            line_no,
            &normalized,
            "dynamic code execution runs attacker-controlled source directly",
            "the line executes text as code instead of keeping it as data",
            "replace dynamic execution with a parser, dispatch table, or typed plugin boundary",
            "code-execution",
        ));
    }

    if is_unsafe_deserialisation_line(&lower) {
        return Some(finding(
            DETECTOR_UNSAFE_DESER,
            "pickle",
            file,
            line_no,
            &normalized,
            "unsafe deserialisation can instantiate attacker-controlled objects",
            "a loader accepts data that can control object construction",
            "use `safe_load` or a schema-based decoder for untrusted input",
            "deserialisation-boundary",
        ));
    }

    if is_shell_dynamic_line(&lower) {
        return Some(finding(
            DETECTOR_SHELL_DYNAMIC,
            "shell=True",
            file,
            line_no,
            &normalized,
            "shell=True or os.system lets input reach a shell",
            "command execution is routed through a shell boundary",
            "pass argv arrays to subprocess and keep shell off",
            "shell-boundary",
        ));
    }

    if is_sql_string_built_line(&lower) {
        return Some(finding(
            DETECTOR_SQL_STRING_BUILT,
            "execute",
            file,
            line_no,
            &normalized,
            "string-built SQL reaches a database sink without parameter binding",
            "the SQL text is assembled inline before the execute/query call",
            "parameterize the statement or move identifier handling through an allowlist",
            "sql-boundary",
        ));
    }

    if is_tls_debug_line(&lower) {
        return Some(finding(
            DETECTOR_TLS_DEBUG,
            "verify=False",
            file,
            line_no,
            &normalized,
            "certificate verification is disabled in runtime code",
            "the request path bypasses TLS verification or trust checks",
            "remove the debug bypass and pin a trusted CA bundle",
            "tls-boundary",
        ));
    }

    None
}

fn advisory_hit_for_line(file: &FileInfo, line_no: usize, line: &str) -> Option<LanguageFinding> {
    let Some(normalized) = normalize_python_line(line) else {
        return None;
    };
    let lower = normalized.to_ascii_lowercase();
    if lower.contains("except exception") || lower.contains("except baseexception") {
        return Some(finding(
            DETECTOR_BROAD_EXCEPT,
            "except exception",
            file,
            line_no,
            &normalized,
            "broad exception handling hides real failures and control flow",
            "the catch-all scope can swallow unrelated errors",
            "catch specific exceptions and keep the failure surface explicit",
            "exception-shape",
        ));
    }
    None
}

fn normalize_python_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with('#') || trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
        return None;
    }
    let without_trailing_comment = trimmed.split('#').next().unwrap_or(trimmed).trim();
    if without_trailing_comment.is_empty() {
        return None;
    }
    Some(
        without_trailing_comment
            .trim_end_matches(';')
            .trim()
            .to_string(),
    )
}

fn is_dynamic_code_line(lower: &str) -> bool {
    lower.contains("eval(")
        || lower.contains("exec(")
        || lower.contains("compile(")
        || lower.contains("exec(open(")
}

fn is_unsafe_deserialisation_line(lower: &str) -> bool {
    if lower.contains("yaml.load(") {
        return !lower.contains("safe_load") && !lower.contains("safeloader");
    }
    [
        "pickle.load",
        "pickle.loads",
        "dill.load",
        "dill.loads",
        "cloudpickle.load",
        "cloudpickle.loads",
        "marshal.load",
        "marshal.loads",
        "shelve.open",
        "joblib.load",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn is_shell_dynamic_line(lower: &str) -> bool {
    lower.contains("shell=true")
        || lower.contains("os.system(")
        || lower.contains("os.popen(")
        || (lower.contains("subprocess.") && lower.contains("shell=true"))
}

fn is_sql_string_built_line(lower: &str) -> bool {
    let has_sink =
        lower.contains("execute(") || lower.contains("executemany(") || lower.contains("query(");
    let has_sql = lower.contains("select ")
        || lower.contains("insert ")
        || lower.contains("update ")
        || lower.contains("delete ")
        || lower.contains("drop ")
        || lower.contains("truncate ");
    let has_dynamic = lower.contains("f\"")
        || lower.contains("f'")
        || lower.contains(".format(")
        || lower.contains(" + ")
        || lower.contains("${");
    has_sink && has_sql && has_dynamic
}

fn is_tls_debug_line(lower: &str) -> bool {
    lower.contains("verify=false")
        || lower.contains("disable_warnings")
        || lower.contains("_create_unverified_context")
}

fn sort_key(a: &LanguageFinding, b: &LanguageFinding) -> std::cmp::Ordering {
    a.path
        .cmp(&b.path)
        .then(a.line.unwrap_or(0).cmp(&b.line.unwrap_or(0)))
        .then(a.matched_term.cmp(&b.matched_term))
        .then(a.problem.cmp(&b.problem))
}

fn finding(
    detector_id: &'static str,
    matched_term: &'static str,
    file: &FileInfo,
    line_no: usize,
    line: &str,
    problem: &str,
    reason: &str,
    agent_fix: &str,
    proof_window: &'static str,
) -> LanguageFinding {
    let snippet = line.trim().chars().take(160).collect::<String>();
    LanguageFinding::new(
        HLT_RULE_ID,
        matched_term,
        file.rel_path.clone(),
        Some(line_no),
        snippet.clone(),
        problem,
        reason,
        agent_fix,
        vec![
            format!("detector={detector_id}"),
            format!("proof-window={proof_window}"),
            format!("snippet={snippet}"),
        ],
    )
}
