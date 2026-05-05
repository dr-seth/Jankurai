use crate::audit::helpers::*;
use crate::audit::scan;
use crate::model::DimensionResult;

pub fn analyze(ctx: &AuditContext) -> DimensionResult {
    let files = product_code_files(ctx);
    if files.is_empty() {
        return make_dim(
            "Code shape and semantic surface",
            if ctx.self_audit { 65 } else { 90 },
            vec!["no authored adopter product code files in scope".into()],
            vec![],
        );
    }
    let mut score = 55;
    let mut evidence = vec![];
    let mut notes = vec![];
    if let Some(file) = largest_file(&files) {
        evidence.push(format!(
            "largest authored code file: {} ({} LOC)",
            file.rel_path, file.line_count
        ));
    }
    if let Some(max) = max_loc(&files) {
        if max > 500 {
            score -= 15;
            evidence.push("code file exceeds 500 LOC".into());
        }
        if max > 1000 {
            score -= 20;
            evidence.push("code file exceeds 1000 LOC".into());
        }
    }
    if files.len() >= 5
        && files.iter().filter(|f| f.line_count <= 300).count() * 10 / files.len() >= 7
    {
        score += 10;
        evidence.push("most code files stay under 300 LOC".into());
    }
    if scan::duplicate_blocks(ctx).is_empty() == false {
        score -= 18;
        evidence.push("duplicate code block marker found".into());
    }
    if !scan::todo_hits(ctx).is_empty() {
        score -= 20;
        evidence.push("TODO/stub marker found".into());
    }
    if scan::fallback_hits(ctx).len() > 1 {
        score -= 18;
        evidence.push("fallback soup marker found".into());
    }
    if !scan::future_hostile_hits(ctx).is_empty() {
        score -= 24;
        evidence.push("future-hostile/dead-language marker found".into());
        notes.push("product/runtime code contains future-hostile or dead-language terms".into());
    }
    if !weak_name_hits(ctx).is_empty() {
        score -= 10;
        evidence.push("weak name marker found".into());
    }
    if !domain_io_hits(ctx).is_empty() {
        score -= 10;
        evidence.push("IO markers found in domain/core files".into());
    }
    let rust_summary = crate::audit::language_rules::rust::summary(ctx);
    if rust_summary.hard_findings > 0 {
        evidence.push(format!(
            "rust bad-behavior hard findings: {}",
            rust_summary.hard_findings
        ));
        notes.push("rust hard behavior is already captured by the language-rule catalog".into());
    } else if rust_summary.advisory_signals > 0 {
        evidence.push(format!(
            "rust bad-behavior advisory signals: {}",
            rust_summary.advisory_signals
        ));
    }
    for (label, hard, advisory) in [
        (
            "sql",
            crate::audit::language_rules::sql::summary(ctx).hard_findings,
            crate::audit::language_rules::sql::summary(ctx).advisory_signals,
        ),
        (
            "typescript",
            crate::audit::language_rules::typescript::summary(ctx).hard_findings,
            crate::audit::language_rules::typescript::summary(ctx).advisory_signals,
        ),
        (
            "docker",
            crate::audit::language_rules::docker::summary(ctx).hard_findings,
            crate::audit::language_rules::docker::summary(ctx).advisory_signals,
        ),
        (
            "python",
            crate::audit::language_rules::python::summary(ctx).hard_findings,
            crate::audit::language_rules::python::summary(ctx).advisory_signals,
        ),
        (
            "ci",
            crate::audit::language_rules::ci::summary(ctx).hard_findings,
            crate::audit::language_rules::ci::summary(ctx).advisory_signals,
        ),
        (
            "git",
            crate::audit::language_rules::git::summary(ctx).hard_findings,
            crate::audit::language_rules::git::summary(ctx).advisory_signals,
        ),
    ] {
        if hard > 0 {
            evidence.push(format!("{label} bad-behavior hard findings: {hard}"));
        } else if advisory > 0 {
            evidence.push(format!("{label} bad-behavior advisory signals: {advisory}"));
        }
    }
    make_dim("Code shape and semantic surface", score, evidence, notes)
}
