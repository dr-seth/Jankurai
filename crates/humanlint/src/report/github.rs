use crate::model::Report;

pub fn render_step_summary(report: &Report) -> String {
    let mut out = String::new();
    use std::fmt::Write;
    let decision = report
        .decision
        .as_ref()
        .map(|decision| decision.status.as_str())
        .unwrap_or("unknown");
    let _ = writeln!(out, "### humanlint");
    let _ = writeln!(out);
    let _ = writeln!(out, "- score: `{}`", report.score);
    let _ = writeln!(out, "- raw score: `{}`", report.raw_score);
    let _ = writeln!(out, "- decision: `{}`", decision);
    let _ = writeln!(out, "- findings: `{}`", report.findings.len());
    if report.ux_qa.artifact.is_some() || report.security_evidence.artifact.is_some() {
        let _ = writeln!(out);
        let _ = writeln!(out, "#### lane artifacts");
        if let Some(art) = &report.ux_qa.artifact {
            let _ = writeln!(
                out,
                "- ux-qa `{}`: reports={} worst={} violations={}",
                art.path, art.report_count, art.worst_decision, art.total_violations
            );
        }
        if let Some(art) = &report.security_evidence.artifact {
            let _ = writeln!(
                out,
                "- security `{}`: exit={} strict={} ran={}/skip={}/fail={}",
                art.path,
                art.envelope_exit_code,
                art.wrapper_strict,
                art.commands_ran,
                art.commands_skipped,
                art.commands_failed
            );
        }
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "#### agent_fix_queue");
    if report.agent_fix_queue.is_empty() {
        let _ = writeln!(out, "No queued fixes.");
    } else {
        for item in &report.agent_fix_queue {
            let _ = writeln!(out, "- [{}] `{}`: {}", item.priority, item.path, item.task);
        }
    }
    out
}
