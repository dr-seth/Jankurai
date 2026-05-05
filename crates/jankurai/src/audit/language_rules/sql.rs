use super::catalog::{
    ConfidencePolicy, Language, LanguageFinding, LanguageRule, Matcher, ProofWindow,
};
use crate::audit::helpers::AuditContext;
use crate::model::FileInfo;

const HLT_RULE_ID: &str = "HLT-030-SQL-BAD-BEHAVIOR";
const DETECTOR_DYNAMIC_SQL: &str = "sql.dynamic-sql";
const DETECTOR_DESTRUCTIVE_MIGRATION: &str = "sql.migration.destructive-no-proof";
const DETECTOR_FULL_TABLE_WRITE: &str = "sql.query.full-table-write";
const DETECTOR_SELECT_STAR: &str = "sql.review.select-star";

const RULES: &[LanguageRule] = &[
    LanguageRule {
        id: DETECTOR_DYNAMIC_SQL,
        language: Language::Sql,
        hlt_rule_id: HLT_RULE_ID,
        severity: "high",
        category: "security",
        lane: "db",
        confidence: ConfidencePolicy::High,
        matcher: Matcher::ContainsAny(&["execute", "query(", "raw(", "concat(", "||", "format("]),
        proof_window: ProofWindow::None,
        problem: "string-built SQL reaches an execution sink without parameter binding",
        fix: "parameterize the statement or use a fixed allowlisted identifier path",
    },
    LanguageRule {
        id: DETECTOR_DESTRUCTIVE_MIGRATION,
        language: Language::Sql,
        hlt_rule_id: HLT_RULE_ID,
        severity: "high",
        category: "security",
        lane: "db",
        confidence: ConfidencePolicy::High,
        matcher: Matcher::ContainsAny(&["drop table", "truncate", "drop column", "disable trigger"]),
        proof_window: ProofWindow::None,
        problem: "destructive migration appears without nearby rollback or backup proof",
        fix: "split the change into a reviewed migration with rollback, backup, and row-count evidence",
    },
    LanguageRule {
        id: DETECTOR_FULL_TABLE_WRITE,
        language: Language::Sql,
        hlt_rule_id: HLT_RULE_ID,
        severity: "high",
        category: "security",
        lane: "db",
        confidence: ConfidencePolicy::High,
        matcher: Matcher::ContainsAny(&["delete from", "update "]),
        proof_window: ProofWindow::None,
        problem: "UPDATE or DELETE without a WHERE clause can rewrite every row",
        fix: "add a WHERE clause or prove the full-table rewrite with a local migration receipt",
    },
    LanguageRule {
        id: DETECTOR_SELECT_STAR,
        language: Language::Sql,
        hlt_rule_id: HLT_RULE_ID,
        severity: "medium",
        category: "advisory",
        lane: "db",
        confidence: ConfidencePolicy::Low,
        matcher: Matcher::ContainsAny(&["select *"]),
        proof_window: ProofWindow::None,
        problem: "unbounded projection obscures the schema contract",
        fix: "name columns explicitly so the query shape stays stable",
    },
];

#[derive(Debug, Clone, Copy, Default)]
pub struct SqlSummary {
    pub hard_findings: usize,
    pub advisory_signals: usize,
}

pub fn catalog() -> &'static [LanguageRule] {
    RULES
}

pub fn summary(ctx: &AuditContext) -> SqlSummary {
    SqlSummary {
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
    for file in sql_files(ctx) {
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
    for file in sql_files(ctx) {
        for (idx, line) in file.text.lines().enumerate() {
            if let Some(hit) = hard_hit_for_line(&file, idx + 1, line, &file.text) {
                out.push(hit);
            }
        }
    }
    out
}

fn sql_files(ctx: &AuditContext) -> Vec<FileInfo> {
    ctx.all_files
        .iter()
        .filter(|file| is_sql_candidate(file))
        .cloned()
        .collect()
}

fn is_sql_candidate(file: &FileInfo) -> bool {
    let rel = file.rel_path.to_ascii_lowercase();
    if file.is_generated || is_excluded_path(&rel) {
        return false;
    }
    matches!(file.suffix.as_str(), ".sql" | ".pgsql" | ".psql") || rel.ends_with("pg_hba.conf")
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

fn hard_hit_for_line(
    file: &FileInfo,
    line_no: usize,
    line: &str,
    full_text: &str,
) -> Option<LanguageFinding> {
    let Some(normalized) = normalize_sql_line(line) else {
        return None;
    };
    let lower = normalized.to_ascii_lowercase();
    if lower.is_empty() {
        return None;
    }

    if is_destructive_migration_line(&lower) && !has_nearby_migration_proof(full_text, line_no) {
        return Some(finding(
            DETECTOR_DESTRUCTIVE_MIGRATION,
            "drop table",
            file,
            line_no,
            &normalized,
            "destructive migration appears without nearby rollback or backup proof",
            "the migration can remove or rewrite data without local evidence of recovery",
            "split the change into a reviewed migration with rollback, backup, and row-count evidence",
            "nearby-proof",
        ));
    }

    if is_full_table_write_line(&lower) {
        return Some(finding(
            DETECTOR_FULL_TABLE_WRITE,
            "update/delete",
            file,
            line_no,
            &normalized,
            "UPDATE or DELETE without a WHERE clause can rewrite every row",
            "the statement reaches a whole-table write path without a row filter",
            "add a WHERE clause or prove the full-table rewrite with a local migration receipt",
            "where-clause",
        ));
    }

    if is_dynamic_sql_line(&lower) {
        return Some(finding(
            DETECTOR_DYNAMIC_SQL,
            "execute",
            file,
            line_no,
            &normalized,
            "string-built SQL reaches an execution sink without parameter binding",
            "a dynamic string or identifier path feeds the SQL execution surface",
            "parameterize the statement or use a fixed allowlisted identifier path",
            "parameter-boundary",
        ));
    }

    None
}

fn advisory_hit_for_line(file: &FileInfo, line_no: usize, line: &str) -> Option<LanguageFinding> {
    let Some(normalized) = normalize_sql_line(line) else {
        return None;
    };
    let lower = normalized.to_ascii_lowercase();
    if lower.contains("select *") {
        return Some(finding(
            DETECTOR_SELECT_STAR,
            "select *",
            file,
            line_no,
            &normalized,
            "unbounded projection obscures the schema contract",
            "the query shape is broader than the consumer contract needs",
            "name columns explicitly so the query shape stays stable",
            "projection-shape",
        ));
    }
    None
}

fn normalize_sql_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("--") || lower.starts_with("/*") || lower.starts_with('*') {
        return None;
    }
    let without_trailing_comment = trimmed.split("--").next().unwrap_or(trimmed).trim();
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

fn is_destructive_migration_line(lower: &str) -> bool {
    lower.contains("drop table")
        || lower.contains("drop index")
        || lower.contains("truncate ")
        || lower.contains("truncate table")
        || lower.contains("drop column")
        || lower.contains("disable trigger")
        || lower.contains("cascade")
}

fn is_full_table_write_line(lower: &str) -> bool {
    (lower.contains("delete from") && !lower.contains(" where "))
        || (lower.contains("update ") && lower.contains(" set ") && !lower.contains(" where "))
}

fn is_dynamic_sql_line(lower: &str) -> bool {
    let has_sink = lower.contains("execute")
        || lower.contains("query(")
        || lower.contains("raw(")
        || lower.contains("sql`")
        || lower.contains("cursor.execute")
        || lower.contains("db.execute")
        || lower.contains("db.query")
        || lower.contains("sequelize.query")
        || lower.contains("knex.raw");
    let has_dynamic = lower.contains("||")
        || lower.contains("concat(")
        || lower.contains("${")
        || lower.contains("f\"")
        || lower.contains("f'")
        || lower.contains(".format(")
        || lower.contains(" + ");
    let has_sql = lower.contains("select ")
        || lower.contains("insert ")
        || lower.contains("update ")
        || lower.contains("delete ")
        || lower.contains("drop ")
        || lower.contains("truncate ");
    let has_safe_binding = lower.contains(" using ")
        || lower.contains("quote_ident")
        || lower.contains("quote_literal")
        || lower.contains("%i")
        || lower.contains("%l");
    has_sink && has_sql && has_dynamic && !has_safe_binding
}

fn has_nearby_migration_proof(text: &str, line_no: usize) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return false;
    }
    let idx = line_no.saturating_sub(1);
    let start = idx.saturating_sub(3);
    let end = (idx + 4).min(lines.len());
    lines[start..end].iter().any(|candidate| {
        let lower = candidate.trim().to_ascii_lowercase();
        lower.contains("rollback")
            || lower.contains("backup")
            || lower.contains("restore")
            || lower.contains("proof")
            || lower.contains("jankurai:migration-safe")
    })
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
