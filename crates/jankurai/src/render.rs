use crate::model::Report;
use crate::report::proof;
use anyhow::Result;
use std::fs;

pub fn write_json(path: &str, content: &str) -> Result<()> {
    if path != "-" {
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path, content)?;
    } else {
        print!("{content}");
    }
    Ok(())
}

pub fn write_markdown(path: &str, content: &str) -> Result<()> {
    if path != "-" {
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path, content)?;
    } else {
        print!("{content}");
    }
    Ok(())
}

pub fn render_markdown(report: &Report) -> String {
    let mut out = String::new();
    use std::fmt::Write;
    let _ = writeln!(out, "# jankurai Repo Score");
    let _ = writeln!(out);
    let _ = writeln!(out, "- Standard: `{}`", report.standard);
    let _ = writeln!(out, "- Auditor: `{}`", report.auditor_version);
    let _ = writeln!(out, "- Schema: `{}`", report.schema_version);
    let _ = writeln!(out, "- Paper edition: `{}`", report.paper_edition);
    let _ = writeln!(out, "- Target stack ID: `{}`", report.target_stack_id);
    let _ = writeln!(out, "- Target stack: `{}`", report.target_stack);
    let _ = writeln!(out, "- Repo: `{}`", report.repo);
    if let Some(run_id) = &report.run_id {
        let _ = writeln!(out, "- Run ID: `{}`", run_id);
    }
    if let Some(started_at) = &report.started_at {
        let _ = writeln!(out, "- Started at: `{}`", started_at);
    }
    if let Some(elapsed_ms) = report.elapsed_ms {
        let _ = writeln!(out, "- Elapsed: `{}` ms", elapsed_ms);
    }
    let _ = writeln!(out, "- Scope: `{}`", report.scope.mode);
    if !report.scope.paths.is_empty() {
        let _ = writeln!(out, "- Changed: `{}`", report.scope.paths.join(", "));
    }
    proof::append_proof_receipts(&mut out, report);
    let _ = writeln!(out, "- Raw score: `{}`", report.raw_score);
    let _ = writeln!(out, "- Final score: `{}`", report.score);
    if let Some(decision) = &report.decision {
        let _ = writeln!(out, "- Decision: `{}`", decision.status);
        let _ = writeln!(out, "- Minimum score: `{}`", decision.minimum_score);
    }
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
    if let Some(art) = &report.ux_qa.artifact {
        let _ = writeln!(out);
        let _ = writeln!(out, "### Ingested UX QA report (`{}`)", art.path);
        let _ = writeln!(out, "- Report count: `{}`", art.report_count);
        let _ = writeln!(out, "- Worst decision: `{}`", art.worst_decision);
        let _ = writeln!(out, "- Total violations: `{}`", art.total_violations);
        let _ = writeln!(
            out,
            "- Summary errors / warnings: `{}` / `{}`",
            art.summary_errors, art.summary_warnings
        );
        let artifact_counts = if art.artifact_counts_by_kind.is_empty() {
            "none".into()
        } else {
            art.artifact_counts_by_kind
                .iter()
                .map(|(kind, count)| format!("{kind}={count}"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let _ = writeln!(out, "- Artifact counts: `{}`", artifact_counts);
        let _ = writeln!(
            out,
            "- Artifact fingerprints: `{}`",
            art.artifact_fingerprint_count
        );
        let _ = writeln!(
            out,
            "- Visual baseline counts: missing=`{}` changed=`{}` review=`{}` block=`{}`",
            art.visual_baseline_missing,
            art.visual_baseline_changed,
            art.visual_baseline_review,
            art.visual_baseline_block
        );
        let _ = writeln!(
            out,
            "- Missing required states: `{}` report(s) `{}`",
            art.reports_missing_required_states,
            if art.missing_state_names.is_empty() {
                "none".into()
            } else {
                art.missing_state_names.join(", ")
            }
        );
        let _ = writeln!(
            out,
            "- Missing required artifacts: `{}` report(s) `{}`",
            art.reports_missing_required_artifacts,
            if art.missing_artifact_kinds.is_empty() {
                "none".into()
            } else {
                art.missing_artifact_kinds.join(", ")
            }
        );
        let _ = writeln!(
            out,
            "- Accessibility violations / incomplete / passes: `{}` / `{}` / `{}`",
            art.accessibility_violation_total,
            art.accessibility_incomplete_total,
            art.accessibility_pass_total
        );
    }
    if let Some(art) = &report.security_evidence.artifact {
        let _ = writeln!(out);
        let _ = writeln!(out, "## Security evidence (ingested)");
        let _ = writeln!(out);
        let _ = writeln!(out, "- Source: `{}`", art.path);
        let _ = writeln!(
            out,
            "- Envelope exit code: `{}` · elapsed: `{}` ms · strict: `{}`",
            art.envelope_exit_code, art.elapsed_ms, art.wrapper_strict
        );
        let _ = writeln!(
            out,
            "- Commands — ran: `{}`, skipped: `{}`, failed: `{}`",
            art.commands_ran, art.commands_skipped, art.commands_failed
        );
        if let Some(ts) = &art.generated_at {
            let _ = writeln!(out, "- Generated at: `{}`", ts);
        }
        if let Some(gh) = &art.git_head {
            let _ = writeln!(out, "- Git HEAD (envelope): `{}`", gh);
        }
    }
    if let Some(art) = &report.boundaries.artifact {
        let _ = writeln!(out);
        let _ = writeln!(out, "## Boundary manifest (ingested)");
        let _ = writeln!(out);
        let _ = writeln!(out, "- Path: `{}`", art.path);
        if let Some(v) = &art.stack_version {
            let _ = writeln!(out, "- Stack: `{}` · version: `{}`", art.stack_id, v);
        } else {
            let _ = writeln!(out, "- Stack: `{}`", art.stack_id);
        }
        let _ = writeln!(
            out,
            "- Queue path counts — adapter: `{}`, event_contract: `{}`, generated_type: `{}`, client_marker: `{}`, streaming_exception: `{}`",
            art.adapter_path_count,
            art.event_contract_path_count,
            art.generated_type_path_count,
            art.client_marker_count,
            art.streaming_exception_count
        );
        let _ = writeln!(out, "- Content fingerprint: `{}`", art.content_fingerprint);
    }
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
            let _ = writeln!(
                out,
                "   Check: `{}` `{}` confidence `{:.2}`",
                finding.check_id, finding.hardness, finding.confidence
            );
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
            let _ = writeln!(out, "   Rerun: `{}`", finding.rerun_command);
            let _ = writeln!(out, "   Fingerprint: `{}`", finding.fingerprint);
            if !finding.evidence.is_empty() {
                let _ = writeln!(out, "   Evidence: {}", finding.evidence.join(", "));
            }
        }
    }
    let _ = writeln!(out);
    if let Some(policy) = &report.policy {
        let _ = writeln!(out, "## Policy");
        let _ = writeln!(out);
        let _ = writeln!(out, "- Policy file: `{}`", policy.path);
        let _ = writeln!(out, "- Minimum score: `{}`", policy.minimum_score);
        let _ = writeln!(out, "- Fail on: `{}`", policy.fail_on.join(", "));
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
