use crate::model::Report;
use anyhow::Result;
use std::fs;

pub fn write_json(path: &str, content: &str) -> Result<()> {
    if path != "-" {
        fs::write(path, content)?;
    } else {
        print!("{content}");
    }
    Ok(())
}

pub fn write_markdown(path: &str, content: &str) -> Result<()> {
    if path != "-" {
        fs::write(path, content)?;
    } else {
        print!("{content}");
    }
    Ok(())
}

pub fn render_markdown(report: &Report) -> String {
    let mut out = String::new();
    use std::fmt::Write;
    let _ = writeln!(out, "# humanlint Repo Score");
    let _ = writeln!(out);
    let _ = writeln!(out, "- Standard: `{}`", report.standard);
    let _ = writeln!(out, "- Auditor: `{}`", report.auditor_version);
    let _ = writeln!(out, "- Schema: `{}`", report.schema_version);
    let _ = writeln!(out, "- Paper edition: `{}`", report.paper_edition);
    let _ = writeln!(out, "- Target stack ID: `{}`", report.target_stack_id);
    let _ = writeln!(out, "- Target stack: `{}`", report.target_stack);
    let _ = writeln!(out, "- Repo: `{}`", report.repo);
    let _ = writeln!(out, "- Scope: `{}`", report.scope.mode);
    if !report.scope.paths.is_empty() {
        let _ = writeln!(out, "- Changed: `{}`", report.scope.paths.join(", "));
    }
    let _ = writeln!(out, "- Raw score: `{}`", report.raw_score);
    let _ = writeln!(out, "- Final score: `{}`", report.score);
    let _ = writeln!(
        out,
        "- Caps applied: `{}`",
        if report.caps_applied.is_empty() {
            "none".into()
        } else {
            report.caps_applied.join(", ")
        }
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "## Hard Rule Caps");
    let _ = writeln!(out);
    let _ = writeln!(out, "| Rule | Max Score | Applied |");
    let _ = writeln!(out, "| --- | ---: | --- |");
    for rule in &report.hard_rules {
        let mark = if report.caps_applied.iter().any(|c| c == &rule.id) {
            "yes"
        } else {
            "no"
        };
        let _ = writeln!(out, "| `{}` | {} | {} |", rule.id, rule.max_score, mark);
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "## Dimensions");
    let _ = writeln!(out);
    let _ = writeln!(out, "| Dimension | Weight | Score | Weighted | Evidence |");
    let _ = writeln!(out, "| --- | ---: | ---: | ---: | --- |");
    for dim in &report.dimensions {
        let evidence = dim
            .evidence
            .iter()
            .take(2)
            .cloned()
            .collect::<Vec<_>>()
            .join("; ");
        let _ = writeln!(
            out,
            "| {} | {} | {} | {:.2} | {} |",
            dim.name, dim.weight, dim.score, dim.weighted_points, evidence
        );
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "## Rendered UX QA");
    let _ = writeln!(out);
    let _ = writeln!(out, "- Web surface: `{}`", report.ux_qa.web_surface);
    let _ = writeln!(
        out,
        "- Layered UX lane: `{}`",
        report.ux_qa.has_rendered_ux_lane
    );
    let _ = writeln!(
        out,
        "- Missing: `{}`",
        if report.ux_qa.missing_categories.is_empty() {
            "none".into()
        } else {
            report.ux_qa.missing_categories.join(", ")
        }
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "## Findings");
    let _ = writeln!(out);
    if report.findings.is_empty() {
        let _ = writeln!(out, "No findings.");
    } else {
        for (idx, finding) in report.findings.iter().enumerate() {
            let loc = if let Some(line) = finding.line {
                format!("{}:{}", finding.path, line)
            } else {
                finding.path.clone()
            };
            let _ = writeln!(
                out,
                "{}. `{}` `{}` `{}`",
                idx + 1,
                finding.severity,
                finding.category,
                loc
            );
            if let Some(rule) = &finding.rule_id {
                let _ = writeln!(out, "   Rule: `{}`", rule);
            }
            if finding.tlr.is_some() || finding.lane.is_some() || finding.owner.is_some() {
                let _ = writeln!(
                    out,
                    "   Route: TLR `{}`, lane `{}`, owner `{}`",
                    finding.tlr.as_deref().unwrap_or("unknown"),
                    finding.lane.as_deref().unwrap_or("unknown"),
                    finding.owner.as_deref().unwrap_or("unmapped"),
                );
            }
            if let Some(url) = &finding.docs_url {
                let _ = writeln!(out, "   Docs: `{}`", url);
            }
            if let Some(term) = &finding.matched_term {
                let _ = writeln!(out, "   Matched term: `{}`", term);
            }
            let _ = writeln!(
                out,
                "   Reason: {}",
                finding.reason.as_deref().unwrap_or(&finding.problem)
            );
            let _ = writeln!(out, "   Fix: {}", finding.agent_fix);
            if !finding.evidence.is_empty() {
                let _ = writeln!(out, "   Evidence: {}", finding.evidence.join(", "));
            }
        }
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "## Agent Fix Queue");
    let _ = writeln!(out);
    if report.agent_fix_queue.is_empty() {
        let _ = writeln!(out, "No queued fixes.");
    } else {
        for (idx, item) in report.agent_fix_queue.iter().enumerate() {
            let rule = item
                .rule_id
                .as_ref()
                .map(|s| format!(" `{}`", s))
                .unwrap_or_default();
            let route = match (&item.tlr, &item.lane) {
                (Some(tlr), Some(lane)) => format!(" `{}`/`{}`", tlr, lane),
                _ => String::new(),
            };
            let _ = writeln!(
                out,
                "{}. `{}`{} `{}` - {}",
                idx + 1,
                item.priority,
                rule,
                item.path,
                item.task
            );
            let _ = writeln!(out, "   Route:{}{}", route, "");
        }
    }
    out
}
